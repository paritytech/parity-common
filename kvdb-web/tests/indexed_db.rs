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

//! IndexedDB tests.

use futures::future::TryFutureExt as _;

use kvdb_web::{Database, KeyValueDB as _};

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn reopen_the_database_with_more_columns() {
	let _ = console_log::init_with_level(log::Level::Trace);

	async fn open_db(col: u32) -> Database {
		Database::open("MyAsyncTest".into(), col).unwrap_or_else(|err| panic!("{}", err)).await
	}

	let db = open_db(1).await;

	// Write a value into the database
	let mut batch = db.transaction();
	batch.put(0, b"hello", b"world");
	db.write_buffered(batch);

	assert_eq!(db.get(0, b"hello").unwrap().unwrap(), b"world");

	// Check the database version
	assert_eq!(db.version(), 1);

	// Close the database
	drop(db);

	// Reopen it again with 3 columns
	let db = open_db(3).await;

	// The value should still be present
	assert_eq!(db.get(0, b"hello").unwrap().unwrap(), b"world");
	assert!(db.get(0, b"trash").unwrap().is_none());

	// The version should be bumped
	assert_eq!(db.version(), 2);
}
