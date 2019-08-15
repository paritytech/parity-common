use futures::compat;
use futures::future::{self, FutureExt as _};

use kvdb_web::{Database, KeyValueDB as _};

use wasm_bindgen_test::*;
use wasm_bindgen::JsValue;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
fn my_async_test() -> impl futures01::Future<Item = (), Error = JsValue> {
	console_log::init_with_level(log::Level::Trace).expect("error initializing log");

	fn open_db() -> impl future::Future<Output = Database> {
		Database::open("MyAsyncTest".into(), 4)
	}

	let fut = open_db().then(|db| {
		// Write a value into the database
		let mut batch = db.transaction();
		batch.put(None, b"hello", b"world");
		db.write_buffered(batch);

		assert_eq!(db.get(None, b"hello").unwrap().unwrap().as_ref(), b"world");

		// Close the database
		drop(db);

		// Reopen it again
		open_db()
	}).map(|db| {
		// The value should still be present
		assert_eq!(db.get(None, b"hello").unwrap().unwrap().as_ref(), b"world");
		assert!(db.get(None, b"trash").unwrap().is_none());

		Ok(())
	});

	compat::Compat::new(fut)
}
