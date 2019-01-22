// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

// TODO: docs
// #![deny(missing_docs)]

use std::path::Path;
use std::io;

use kvdb::{KeyValueDB, DBTransaction, DBValue, DBOp};
use lmdb::{
	Environment, Database, DatabaseFlags,
	Transaction, RoTransaction, RwTransaction,
	Iter as LmdbIter, Cursor, RoCursor,
	Error, WriteFlags,
};

use owning_ref::OwningHandle;
use log::warn;

fn other_io_err<E>(e: E) -> io::Error where E: ToString {
	io::Error::new(io::ErrorKind::Other, e.to_string())
}

/// An LMDB `Environment` is a collection of one or more DBs,
/// along with transactions and iterators.
pub struct EnvironmentWithDatabases {
	// Transactions are atomic across all DBs in an `Environment`.
	env: Environment,
	// We use one DB per column.
	// `Database` is essentially a `c_int` (a `Copy` type).
	dbs: Vec<Database>,
}

fn open_or_create_db(env: &Environment, col: u32) -> io::Result<Database> {
	let db_name = format!("col{}", col);
	env.create_db(Some(&db_name[..]), DatabaseFlags::default()).map_err(other_io_err)
}

// TODO: memory management
impl EnvironmentWithDatabases {
	/// Opens an environment path. Creates if it does not exist.
	/// `columns` is a number of non-default columns.
	pub fn open(path: &str, columns: u32) -> io::Result<Self> {
		// account for the default column
		let columns = columns + 1;
		let path = Path::new(path);

		let mut env_builder = Environment::new();
		env_builder.set_max_dbs(columns);

		let env = env_builder.open(&path).map_err(other_io_err)?;

		let mut dbs = Vec::with_capacity(columns as usize);
		for col in 0..columns {
			let db = open_or_create_db(&env, col)?;
			dbs.push(db);
		}

		Ok(Self {
			env,
			dbs,
		})
	}

	fn ro_txn(&self) -> io::Result<RoTransaction> {
		self.env.begin_ro_txn().map_err(other_io_err)
	}

	fn rw_txn(&self) -> io::Result<RwTransaction> {
		self.env.begin_rw_txn().map_err(other_io_err)
	}

	fn column_to_db(&self, col: Option<u32>) -> Database {
		let col = col.map_or(0, |c| (c as usize + 1));
		self.dbs[col]
	}

	pub fn get(&self, col: Option<u32>, key: &[u8]) -> io::Result<Option<DBValue>> {
		let ro_txn = self.ro_txn()?;
		let db = self.column_to_db(col);

		let result = ro_txn.get(db, &key)
			.map(DBValue::from_slice);

		match result {
			Ok(value) => Ok(Some(value)),
			Err(Error::NotFound) => Ok(None),
			Err(e) => Err(other_io_err(e)),
		}
	}

	pub fn write_buffered(&self, txn: DBTransaction) {
		// TODO: this method actually flushes the data to disk.
		//       Shall we use `NO_SYNC` (doesn't flush, but a system crash can corrupt the database)?
		if let Err(e) = self.write(txn) {
			warn!(target: "lmdb", "error while writing a transaction {:?}", e);
		}
	}

	pub fn write(&self, transaction: DBTransaction) -> io::Result<()> {
		let mut rw_txn = self.rw_txn()?;

		for op in transaction.ops {
			match op {
				DBOp::Insert { col, key, value } => {
					debug_assert!(key.len() < 512, "lmdb: MDB_MAXKEYSIZE is 511");
					let db = self.column_to_db(col);
					rw_txn.put(db, &key, &value, WriteFlags::empty()).map_err(other_io_err)?;
				},
				DBOp::Delete { col, key } => {
					let db = self.column_to_db(col);
					rw_txn.del(db, &key, None).map_err(other_io_err)?;
				}
			}
		}

		rw_txn.commit().map_err(other_io_err)
	}

	pub fn flush(&self) -> io::Result<()> {
		// TODO: this only make sense for `NO_SYNC`.
		self.env.sync(true).map_err(other_io_err)
	}

	pub fn iter<'env>(&'env self, col: Option<u32>) -> Option<impl Iterator<Item=(Box<[u8]>, Box<[u8]>)> + 'env> {
		// TODO: how to handle errors properly?
		let ro_txn = self.ro_txn().ok()?;
		let db = self.column_to_db(col);

		// TODO: is there a better way to implement an iterator?
		// The brrwchk complains (because of ro_txn lifetime, rightly so)
		// if we return just Iter.
		// I'm open to any suggestions.
		Some(DatabaseIterator {
			inner: OwningHandle::new_with_fn(
				Box::new(ro_txn),
				move |txn| {
					let txn = unsafe { txn.as_ref().expect("can't be null; qed") };
					let mut cursor = txn.open_ro_cursor(db).expect("lmdb: failed to open a cursor");
					let iter = cursor.iter();
					Box::new(Iter {
						iter,
						cursor,
					})
				}
			),
		})
	}

	fn get_by_prefix(&self, col: Option<u32>, prefix: &[u8]) -> Option<Box<[u8]>> {
		self.iter_from_prefix(col, prefix).and_then(|mut iter| {
			match iter.next() {
				Some((k, v)) => if k.starts_with(prefix) { Some(v) } else { None },
				_ => None
			}
		})
	}

	fn iter_from_prefix(&self, col: Option<u32>, prefix: &[u8]) -> Option<DatabaseIterator> {
		let ro_txn = self.ro_txn().ok()?;
		let db = self.column_to_db(col);

		Some(DatabaseIterator {
			inner: OwningHandle::new_with_fn(
				Box::new(ro_txn),
				move |txn| {
					let txn = unsafe { txn.as_ref().expect("can't be null; qed") };
					let mut cursor = txn.open_ro_cursor(db).expect("lmdb: failed to open a cursor");
					let iter = cursor.iter_from(prefix);
					Box::new(Iter {
						iter,
						cursor,
					})
				}
			),
		})
	}

	fn restore(&self, _new_db: &str) -> io::Result<()> {
		unimplemented!("TODO: figure out a way")
	}
}

struct Iter<'env> {
    iter: LmdbIter<'env>,
    cursor: RoCursor<'env>,
}

impl<'env> Iterator for Iter<'env> {
    type Item = Result<(&'env [u8], &'env [u8]), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'env> Drop for Iter<'env> {
	fn drop(&mut self) {
		drop(&mut self.cursor);
	}
}

struct DatabaseIterator<'env> {
	// TODO: does autoderived Drop work properly?
	inner: OwningHandle<Box<RoTransaction<'env>>, Box<Iter<'env>>>,
}


impl<'env> Iterator for DatabaseIterator<'env> {
	type Item = (Box<[u8]>, Box<[u8]>);

	fn next(&mut self) -> Option<Self::Item> {
		use std::ops::DerefMut;

		// TODO: panic instead of silencing errors?
		match self.inner.deref_mut().next().and_then(Result::ok) {
			Some((key, value)) => {
				Some((
					key.to_vec().into_boxed_slice(),
					value.to_vec().into_boxed_slice(),
				))
			},
			_ => None,
		}
	}
}

// Duplicate declaration of methods here to avoid trait import in certain existing cases
// at time of addition.
impl KeyValueDB for EnvironmentWithDatabases {
	fn get(&self, col: Option<u32>, key: &[u8]) -> io::Result<Option<DBValue>> {
		EnvironmentWithDatabases::get(self, col, key)
	}

	fn get_by_prefix(&self, col: Option<u32>, prefix: &[u8]) -> Option<Box<[u8]>> {
		EnvironmentWithDatabases::get_by_prefix(self, col, prefix)
	}

	fn write_buffered(&self, transaction: DBTransaction) {
		EnvironmentWithDatabases::write_buffered(self, transaction)
	}

	fn write(&self, transaction: DBTransaction) -> io::Result<()> {
		EnvironmentWithDatabases::write(self, transaction)
	}

	fn flush(&self) -> io::Result<()> {
		EnvironmentWithDatabases::flush(self)
	}

	fn iter<'a>(&'a self, col: Option<u32>) -> Box<Iterator<Item=(Box<[u8]>, Box<[u8]>)> + 'a> {
		let unboxed = EnvironmentWithDatabases::iter(self, col);
		Box::new(unboxed.into_iter().flat_map(|inner| inner))
	}

	fn iter_from_prefix<'a>(&'a self, col: Option<u32>, prefix: &'a [u8])
		-> Box<Iterator<Item=(Box<[u8]>, Box<[u8]>)> + 'a>
	{
		let unboxed = EnvironmentWithDatabases::iter_from_prefix(self, col, prefix);
		Box::new(unboxed.into_iter().flat_map(|inner| inner))
	}

	fn restore(&self, new_db: &str) -> io::Result<()> {
		EnvironmentWithDatabases::restore(self, new_db)
	}
}

impl Drop for EnvironmentWithDatabases {
	fn drop(&mut self) {
		if let Err(error) = self.flush() {
			warn!(target: "lmdb", "database flush failed: {}", error);
		}
	}
}
