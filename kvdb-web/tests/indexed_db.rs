// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! IndexedDB tests.

use futures::future::TryFutureExt as _;

use kvdb_shared_tests as st;
use kvdb_web::{Database, KeyValueDB as _};

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

async fn open_db(col: u32, name: &str) -> Database {
	Database::open(name.into(), col).unwrap_or_else(|err| panic!("{}", err)).await
}

#[wasm_bindgen_test]
async fn get_fails_with_non_existing_column() {
	let db = open_db(1, "get_fails_with_non_existing_column").await;
	st::test_get_fails_with_non_existing_column(&db).unwrap()
}

#[wasm_bindgen_test]
async fn put_and_get() {
	let db = open_db(1, "put_and_get").await;
	st::test_put_and_get(&db).unwrap()
}

#[wasm_bindgen_test]
async fn delete_and_get() {
	let db = open_db(1, "delete_and_get").await;
	st::test_delete_and_get(&db).unwrap()
}

#[wasm_bindgen_test]
async fn iter() {
	let db = open_db(1, "iter").await;
	st::test_iter(&db).unwrap()
}

#[wasm_bindgen_test]
async fn iter_from_prefix() {
	let db = open_db(1, "iter_from_prefix").await;
	st::test_iter_from_prefix(&db).unwrap()
}

#[wasm_bindgen_test]
async fn complex() {
	let db = open_db(1, "complex").await;
	st::test_complex(&db).unwrap()
}

#[wasm_bindgen_test]
async fn reopen_the_database_with_more_columns() {
	let _ = console_log::init_with_level(log::Level::Trace);

	let db = open_db(1, "reopen_the_database_with_more_columns").await;

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
	let db = open_db(3, "reopen_the_database_with_more_columns").await;

	// The value should still be present
	assert_eq!(db.get(0, b"hello").unwrap().unwrap(), b"world");
	assert!(db.get(0, b"trash").unwrap().is_none());

	// The version should be bumped
	assert_eq!(db.version(), 2);
}
