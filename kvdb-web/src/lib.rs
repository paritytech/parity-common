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

//! A key-value database for use in browsers
//!
//! Writes data both into memory and IndexedDB, reads from the IndexedDB
//! on `open`.


mod error;
mod indexed_db;

use std::io;
use std::rc::Rc;
use std::sync::Mutex;
use kvdb::{DBValue, DBTransaction};
use kvdb_memorydb::{InMemory, self as in_memory};
use send_wrapper::SendWrapper;

pub use error::Error;
pub use kvdb::KeyValueDB;

use futures::prelude::*;

use web_sys::IdbDatabase;

pub struct Database {
	name: String,
	version: u32,
	columns: u32,
	in_memory: InMemory,
	indexed_db: Mutex<SendWrapper<IdbDatabase>>,
}

// The default column is represented as `None`.
type Column = Option<u32>;

fn number_to_column(col: u32) -> Column {
	col.checked_sub(1)
}


impl Database {
	/// Opens the database with the given name, version
	/// and the specified number of columns (not including the default one).
	/// Note, that it's not possible to open a database with a version
	/// lower than it was opened previously.
	pub fn open(name: String, version: u32, columns: u32) -> impl Future<Output = Result<Database, error::Error>> {
		let open_request = indexed_db::open(name.as_str(), version, columns);
		// populate the in_memory db from the IndexedDB
		open_request.then(move |db| {
			let db = match db {
				Ok(db) => db,
				Err(err) => return future::Either::Right(future::err(err)),
			};

			let rc = Rc::new(db);
			let weak = Rc::downgrade(&rc);
			// read the columns from the IndexedDB
			future::Either::Left(stream::iter(0..=columns).map(move |n| {
				let db = weak.upgrade().expect("rc should live at least as long; qed");
				indexed_db::idb_cursor(&db, n).fold(DBTransaction::new(), move |mut txn, (key, value)| {
					let column = number_to_column(n);
					txn.put_vec(column, key.as_ref(), value);
					future::ready(txn)
				})
			// write each column into memory
			}).fold(in_memory::create(columns), |m, txn| {
				txn.then(|txn| {
					m.write_buffered(txn);
					future::ready(m)
				})
			}).then(move |in_memory| future::ok(Database {
				name,
				version,
				columns,
				in_memory,
				indexed_db: Mutex::new(SendWrapper::new(
					Rc::try_unwrap(rc).expect("should have only 1 ref at this point; qed")
				)),
			})))
		})
	}

	/// Get the database name.
	pub fn name(&self) -> &str {
		self.name.as_str()
	}

	/// Get the database version.
	pub fn version(&self) -> u32 {
		self.version
	}
}

impl Drop for Database {
	fn drop(&mut self) {
		if let Ok(db) = self.indexed_db.lock() {
			db.close();
		}
	}
}

impl KeyValueDB for Database {
	fn get(&self, col: Option<u32>, key: &[u8]) -> io::Result<Option<DBValue>> {
		self.in_memory.get(col, key)
	}

	fn get_by_prefix(&self, col: Option<u32>, prefix: &[u8]) -> Option<Box<[u8]>> {
		self.in_memory.get_by_prefix(col, prefix)
	}

	fn write_buffered(&self, transaction: DBTransaction) {
		if let Ok(guard) = self.indexed_db.lock() {
			let _ = indexed_db::idb_commit_transaction(&*guard, &transaction, self.columns);
		}
		self.in_memory.write_buffered(transaction);
	}

	fn flush(&self) -> io::Result<()> {
		Ok(())
	}

	// NOTE: clones the whole db
	fn iter<'a>(&'a self, col: Option<u32>) -> Box<dyn Iterator<Item=(Box<[u8]>, Box<[u8]>)> + 'a> {
		self.in_memory.iter(col)
	}

	// NOTE: clones the whole db
	fn iter_from_prefix<'a>(&'a self, col: Option<u32>, prefix: &'a [u8])
		-> Box<dyn Iterator<Item=(Box<[u8]>, Box<[u8]>)> + 'a>
	{
		self.in_memory.iter_from_prefix(col, prefix)
	}

	// NOTE: not supported
	fn restore(&self, _new_db: &str) -> std::io::Result<()> {
		Err(io::Error::new(io::ErrorKind::Other, "Not supported yet"))
	}
}
