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


mod indexed_db;

use std::io;
use std::rc::Rc;
use kvdb::{DBValue, DBTransaction};
use kvdb_memorydb::{InMemory, self as in_memory};
use send_wrapper::SendWrapper;

pub use kvdb::KeyValueDB;

use futures::prelude::*;

use web_sys::IdbDatabase;

pub struct Database {
	name: String,
	columns: u32,
	in_memory: InMemory,
	indexed_db: MakeSync<SendWrapper<IdbDatabase>>,
}

// WARNING: this is UNSAFE for the current implementation
// and relies on WASM being single-threaded atm.
struct MakeSync<T>(T);

unsafe impl<T> Sync for MakeSync<T> {}

impl<T> ::std::ops::Deref for MakeSync<T> {
	type Target = T;

	fn deref(&self) -> &T {
		&self.0
	}
}

impl<T> From<T> for MakeSync<T> {
	fn from(data: T) -> MakeSync<T> {
		MakeSync(data)
	}
}

// The default column is represented as `None`.
type Column = Option<u32>;

fn number_to_column(col: u32) -> Column {
	col.checked_sub(1)
}


impl Database {
	/// Opens the database with the given name
	/// and the specified number of columns (not including the default one).
	pub fn open(name: String, columns: u32) -> impl Future<Output = Database> {
		let open_request = indexed_db::open(name.as_str(), columns);
		// populate the in_memory db from the IndexedDB
		open_request.then(move |db| {
			let rc = Rc::new(db);
			let weak = Rc::downgrade(&rc);
			// read the columns from the IndexedDB
			stream::iter(0..=columns).map(move |n| {
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
			}).then(move |in_memory| future::ready(Database {
				name,
				columns,
				in_memory,
				indexed_db: MakeSync::from(SendWrapper::new(
					Rc::try_unwrap(rc).expect("should have only 1 ref at this point; qed")
				)),
			}))
		})
	}

	/// Get the database name.
	pub fn name(&self) -> &str {
		self.name.as_str()
	}
}

impl Drop for Database {
	fn drop(&mut self) {
		self.indexed_db.close()
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
		let _ = indexed_db::idb_commit_transaction(&self.indexed_db, &transaction, self.columns);
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
