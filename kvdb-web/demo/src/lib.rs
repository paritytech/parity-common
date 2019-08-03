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

//! Kvdb demo in the browser
//!
//! See `README.md` for installation instructions.
//! You can make any changes to the code
//! and observe the changes without the need to refresh the page
//! thanks to Live Reloading.

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

	drop(db);
	let db = Database::open("hello".into(), 1);

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
