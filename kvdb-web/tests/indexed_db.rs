use futures::compat;
use futures::future::{self, FutureExt as _, TryFutureExt as _};

use kvdb_web::{Database, KeyValueDB as _};

use wasm_bindgen_test::*;
use wasm_bindgen::JsValue;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
fn reopen_the_database_with_more_columns() -> impl futures01::Future<Item = (), Error = JsValue> {
	let _ = console_log::init_with_level(log::Level::Trace);

	fn open_db(col: u32) -> impl future::Future<Output = Database> {
		Database::open("MyAsyncTest".into(), col, col)
			.unwrap_or_else(|err| panic!("{}", err))
	}

	let fut = open_db(1).then(|db| {
		// Write a value into the database
		let mut batch = db.transaction();
		batch.put(None, b"hello", b"world");
		db.write_buffered(batch);

		assert_eq!(db.get(None, b"hello").unwrap().unwrap().as_ref(), b"world");

		// Check the database version
		assert_eq!(db.version(), 1);

		// Close the database
		drop(db);

		// Reopen it again with 2 columns
		open_db(2)
	}).map(|db| {
		// The value should still be present
		assert_eq!(db.get(None, b"hello").unwrap().unwrap().as_ref(), b"world");
		assert!(db.get(None, b"trash").unwrap().is_none());

		// Check the database version again
		assert_eq!(db.version(), 2);

		Ok(())
	});

	compat::Compat::new(fut)
}
