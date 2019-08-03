#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;
use kvdb_web::*;

macro_rules! console_log {
	($($t:tt)*) => (console::log_1(&format!($($t)*).into()))
}

#[wasm_bindgen(start)]
pub fn run() {
	let db = Database::open("hello".into(), 1);

	insert(&db, b"hello", b"world");
	console_log!("DB get {:?}", get(&db, b"hello"));

	console_log!("DB get {:?}", get(&db, b"hello"));
	console_log!("DB get {:?}", get(&db, b"trash"));
}


fn insert(db: &Database, key: &[u8], value: &[u8]) {
	let mut batch = DBTransaction::new();

	batch.put(None, key, value);

	db.write_buffered(batch);
}

fn get(db: &Database, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
	KeyValueDB::get(db, None, key).map(|v| v.map(|v| v.into_vec()))
		.map_err(|e| format!("{:?}", e).into())
}
