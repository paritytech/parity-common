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

//! KeyValueDB implementation backed by [LMDB](http://www.lmdb.tech/doc/).
//!
//! ## Memory usage
//!
//! RSS measurement may report a process as having an entire database resident,
//! but don't be alarmed. LMDB uses pages in file-backed memory maps (OS page cache),
//! which can be reclaimed by the OS at any moment
//! as long as the pages in the map are clean.
//!
//! ## Limitations
//!
//! - Max key size is 511 bytes (compile time constant `MDB_MAXKEYSIZE`).
//! - Max threads performing read-only transactions default to 126.
//! - There is only one active writer allowed at any point of time,
//! other writers will be blocked until that active writer aborts/commits.


#![deny(missing_docs)]

use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::io;

use kvdb::{
	DBValue, NumColumns,
	TransactionHandler, IterationHandler, OpenHandler,
	ReadTransaction, WriteTransaction,
};
use lmdb::{
	Environment, Database as LmdbDatabase, DatabaseFlags,
	Transaction, RoTransaction, RwTransaction,
	Iter as LmdbIter, Cursor, RoCursor,
	Error, WriteFlags,
};

use owning_ref::OwningHandle;

pub use kvdb::DatabaseWithCache;

fn other_io_err<E>(e: E) -> io::Error where E: ToString {
	io::Error::new(io::ErrorKind::Other, e.to_string())
}

type KeyValuePair = (Box<[u8]>, Box<[u8]>);

/// Lmdb config.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DatabaseConfig {
	/// Number of columns in the database.
	num_columns: u32,
}

impl DatabaseConfig {
	/// Create a new database config.
	pub fn new(num_columns: u32) -> Self {
		Self {
			num_columns,
		}
	}
}

impl NumColumns for DatabaseConfig {
	fn num_columns(&self) -> usize {
		self.num_columns as usize
	}
}

impl OpenHandler<EnvironmentWithDatabases> for EnvironmentWithDatabases {
	type Config = DatabaseConfig;

	fn open(config: &Self::Config, path: &str) -> io::Result<Self> {
		Self::open(&Path::new(path), config.num_columns)
	}
}

impl<'a> IterationHandler for &'a EnvironmentWithDatabases {
	type Iterator = IterWithTxn<'a>;

	fn iter(&self, col: usize) -> Self::Iterator {
		// TODO: how to handle errors properly?
		let ro_txn = self.ro_txn().expect("lmdb: transaction creation failed");
		let db = self.dbs[col];

		IterWithTxn {
			inner: OwningHandle::new_with_fn(Box::new(ro_txn), move |txn| {
				let txn = unsafe { txn.as_ref().expect("can't be null; qed") };
				let mut cursor = txn.open_ro_cursor(db).expect("lmdb: failed to open a cursor");
				let iter = cursor.iter();
				DerefWrapper(Iter { iter, _cursor: cursor })
			}),
		}
	}

	fn iter_from_prefix(&self, col: usize, prefix: &[u8]) -> Self::Iterator {
		// TODO: how to handle errors properly?
		let ro_txn = self.ro_txn().expect("lmdb: transaction creation failed");
		let db = self.dbs[col];

		IterWithTxn {
			inner: OwningHandle::new_with_fn(Box::new(ro_txn), move |txn| {
				let txn = unsafe { txn.as_ref().expect("can't be null; qed") };
				let mut cursor = txn.open_ro_cursor(db).expect("lmdb: failed to open a cursor");
				let iter = cursor.iter_from(prefix);
				DerefWrapper(Iter { iter, _cursor: cursor })
			}),
		}
	}
}

impl TransactionHandler for EnvironmentWithDatabases {
	// TODO: how to handle errors here (expect)?
	fn read_transaction<'a>(&'a self) -> Box<ReadTransaction + 'a> {
		Box::new(LmdbReadTransaction {
			inner: self.ro_txn().expect("lmdb: ro transaction creation failed"),
			dbs: &self.dbs[..],
		})
	}

	fn write_transaction<'a>(&'a self) -> Box<WriteTransaction + 'a> {
		Box::new(LmdbWriteTransaction {
			inner: self.rw_txn().expect("lmdb: rw transaction creation failed"),
			dbs: &self.dbs[..],
		})
	}
}

struct LmdbWriteTransaction<'a> {
	inner: RwTransaction<'a>,
	dbs: &'a [LmdbDatabase],
}

impl<'a> WriteTransaction for LmdbWriteTransaction<'a> {
	fn put(&mut self, c: usize, key: &[u8], value: &[u8]) -> io::Result<()> {
		debug_assert!(key.len() < 512, "lmdb: MDB_MAXKEYSIZE is 511");
		let db = self.dbs[c];
		self.inner.put(db, &key, &value, WriteFlags::empty()).map_err(other_io_err)
	}

	fn delete(&mut self, c: usize, key: &[u8]) -> io::Result<()> {
		let db = self.dbs[c];
		self.inner.del(db, &key, None).map_err(other_io_err)
	}

	fn commit(self: Box<Self>) -> io::Result<()> {
		self.inner.commit().map_err(other_io_err)
	}
}

struct LmdbReadTransaction<'a> {
	inner: RoTransaction<'a>,
	dbs: &'a [LmdbDatabase],
}

impl<'a> ReadTransaction for LmdbReadTransaction<'a> {
	fn get(self: Box<Self>, c: usize, key: &[u8]) -> io::Result<Option<DBValue>> {
		let db = self.dbs[c];
		let result = self.inner.get(db, &key).map(DBValue::from_slice);

		match result {
			Ok(value) => Ok(Some(value)),
			Err(Error::NotFound) => Ok(None),
			Err(e) => Err(other_io_err(e)),
		}
	}
}

/// Key-Value database.
pub type Database = DatabaseWithCache<EnvironmentWithDatabases>;

/// An LMDB `Environment` is a collection of one or more DBs,
/// along with transactions and iterators.
#[derive(Debug)]
pub struct EnvironmentWithDatabases {
	// Transactions are atomic across all DBs in an `Environment`.
	env: Environment,
	// We use one DB per column.
	// `LmdbDatabase` is essentially a `c_int` (a `Copy` type).
	dbs: Vec<LmdbDatabase>,
}

fn open_or_create_db(env: &Environment, col: u32) -> io::Result<LmdbDatabase> {
	let db_name = format!("col{}", col);
	env.create_db(Some(&db_name[..]), DatabaseFlags::default()).map_err(other_io_err)
}

impl EnvironmentWithDatabases {
	fn open(path: &Path, columns: u32) -> io::Result<Self> {
		// account for the default column
		let columns = columns + 1;
		let mut env_builder = Environment::new();
		env_builder.set_max_dbs(columns);
		// TODO: this would fail on 32-bit systems
		// use autoresizing https://github.com/mozilla/rkv/pull/132
		let terabyte: usize = 1 << 40;
		env_builder.set_map_size(terabyte);

		let env = env_builder.open(path).map_err(other_io_err)?;

		let mut dbs = Vec::with_capacity(columns as usize);
		for col in 0..columns {
			let db = open_or_create_db(&env, col)?;
			dbs.push(db);
		}

		Ok(Self { env, dbs })
	}

	fn ro_txn(&self) -> io::Result<RoTransaction> {
		self.env.begin_ro_txn().map_err(other_io_err)
	}

	fn rw_txn(&self) -> io::Result<RwTransaction> {
		self.env.begin_rw_txn().map_err(other_io_err)
	}
}

struct Iter<'env> {
	iter: LmdbIter<'env>,
	// we need to drop it after LmdbIter
	_cursor: RoCursor<'env>,
}

impl<'env> Iterator for Iter<'env> {
	type Item = Result<(&'env [u8], &'env [u8]), Error>;

	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next()
	}
}

struct DerefWrapper<T>(T);

impl<T> Deref for DerefWrapper<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> DerefMut for DerefWrapper<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

// TODO: how to avoid boxing?
/// Lmdb iterator.
pub struct IterWithTxn<'env> {
	inner: OwningHandle<
		Box<RoTransaction<'env>>,
		DerefWrapper<Iter<'env>>,
	>,
}

impl<'env> Iterator for IterWithTxn<'env> {
	type Item = KeyValuePair;

	fn next(&mut self) -> Option<Self::Item> {
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

#[cfg(test)]
mod test {
	use super::*;
	use tempdir::TempDir;
	use kvdb::KeyValueDB;

	const KEY_1: &[u8; 4] = b"key1";
	const KEY_2: &[u8; 4] = b"key2";
	const KEY_3: &[u8; 4] = b"key3";

	fn test_implements_kvdb<T: KeyValueDB>(_t: T) {}

	#[test]
	fn test_lmdb_implements_kvdb() {
		let db = setup_db("lmdb");
		test_implements_kvdb(db);
	}


	fn setup_db(name: &str) -> Database {
		let tempdir = TempDir::new(name).unwrap();
		let num_columns = 1;
		let config = DatabaseConfig {
			num_columns,
		};
		let db = Database::open(&config, tempdir.path().to_str().unwrap()).unwrap();

		let mut batch = db.transaction();
		batch.put(None, KEY_1, b"cat");
		batch.put(None, KEY_2, b"dog");
		db.write(batch).unwrap();
		db
	}

	#[test]
	fn test_get() {
		let db = setup_db("test_get");
		assert_eq!(&*db.get(None, KEY_1).unwrap().unwrap(), b"cat");
		assert_eq!(&*db.get(None, KEY_2).unwrap().unwrap(), b"dog");
		assert!(db.get(None, KEY_3).unwrap().is_none());
	}

	#[test]
	fn test_iter() {
		let db = setup_db("test_iter");
		let contents: Vec<_> = db.iter(None).collect();
		assert_eq!(contents.len(), 2);
		assert_eq!(&*contents[0].0, &*KEY_1);
		assert_eq!(&*contents[0].1, b"cat");
		assert_eq!(&*contents[1].0, &*KEY_2);
		assert_eq!(&*contents[1].1, b"dog");
	}

	#[test]
	fn test_delete() {
		let db = setup_db("test_delete");
		let mut batch = db.transaction();
		batch.delete(None, KEY_1);
		db.write(batch).unwrap();

		assert!(db.get(None, KEY_1).unwrap().is_none());

		let mut batch = db.transaction();
		batch.put(None, KEY_1, b"cat");
		db.write(batch).unwrap();

		assert_eq!(&*db.get(None, KEY_1).unwrap().unwrap(), b"cat");

		let mut transaction = db.transaction();
		transaction.put(None, KEY_3, b"elephant");
		transaction.delete(None, KEY_1);
		db.write(transaction).unwrap();

		assert!(db.get(None, KEY_1).unwrap().is_none());
		assert_eq!(&*db.get(None, KEY_3).unwrap().unwrap(), b"elephant");

		assert_eq!(db.iter(None).collect::<Vec<_>>().len(), 2);
	}

	#[test]
	fn test_prefixed() {
		let db = setup_db("test_prefixed");
		let mut transaction = db.transaction();
		transaction.put(None, KEY_3, b"elephant");
		transaction.delete(None, KEY_1);
		db.write(transaction).unwrap();

		assert_eq!(&*db.get_by_prefix(None, KEY_3).unwrap(), b"elephant");
		assert_eq!(&*db.get_by_prefix(None, KEY_2).unwrap(), b"dog");

		const KEY_4: &[u8; 12] = b"prefixed_key";
		const KEY_5: &[u8; 20] = b"prefixed_another_key";

		let mut batch = db.transaction();
		batch.put(Some(0), KEY_4, b"monkey");
		batch.put(Some(0), KEY_5, b"koala");
		db.write(batch).unwrap();

		assert_eq!(&*db.get_by_prefix(Some(0), b"prefixed").unwrap(), b"koala");
		assert_eq!(&*db.get_by_prefix(Some(0), b"prefixed_k").unwrap(), b"monkey");

		let contents: Vec<_> = db.iter_from_prefix(None, b"key").collect();
		assert_eq!(contents.len(), 2);
		assert_eq!(&*contents[0].0, &*KEY_2);
		assert_eq!(&*contents[0].1, b"dog");
		assert_eq!(&*contents[1].0, &*KEY_3);
		assert_eq!(&*contents[1].1, b"elephant");
	}

	#[test]
	#[should_panic(expected = "index out of bounds: the len is 2 but the index is 2")]
	fn test_column_out_of_range() {
		let db = setup_db("test_column_out_of_range");
		let _ = db.get(Some(1), KEY_1).unwrap();
	}
}
