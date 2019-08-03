//! TODO module docs, license

#![cfg(target_arch = "wasm32")]

mod hex;

pub use kvdb::{DBValue, DBTransaction, DBOp, KeyValueDB};
pub use wasm_bindgen::{JsCast, UnwrapThrowExt};
pub use web_sys::console;

use wasm_bindgen::prelude::*;
use web_sys::Storage;

use std::collections::HashMap;
use std::io;
use std::sync::{Arc, RwLock};

use serde::{Serialize, Deserialize};
use crate::hex::BytesHexEncoding;


type Key = BytesHexEncoding;
type Value = BytesHexEncoding;

macro_rules! console_log {
	($($t:tt)*) => (console::log_1(&format!($($t)*).into()))
}

macro_rules! console_warn {
	($($t:tt)*) => (console::warn_1(&format!($($t)*).into()))
}


#[derive(Clone, Serialize, Deserialize)]
struct Columns {
	// A column number is represented as an index in the vec.
	columns: Vec<HashMap<Key, Value>>,
}

impl Columns {
	fn new(columns: u32) -> Self {
		Self {
			columns: vec![Default::default(); columns as usize],
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct Database {
	name: String,
	columns: RwLock<Columns>,
}


const RWLOCK_NO_POISONING_PROOF: &str = "RwLock poisoning can't happen in WASM; qed";

impl Database {
	/// Opens the database with the specified number of columns.
	pub fn open(name: String, columns: u32) -> Arc<Database> {
		let window = web_sys::window().expect_throw("are we in a browser?");
		let local_storage = window.local_storage()
			.expect_throw("localStorage should be supported in your browser")
			.unwrap_throw();

		let mut cols: Option<Columns> = None;
		if let Ok(Some(raw)) = local_storage.get_item(name.as_str()) {
			console_log!("Reading the database from localStorage with size {}", raw.len());
			let result: Result<Columns, _> = serde_json::from_str(raw.as_str());
			match result {
				Ok(db) => {
					cols = Some(db);
				},
				Err(e) => {
					console_warn!("Error deserializing the db: {}", e);
				}
			}
		}

		let db = Arc::new(Database {
			name,
			// +1 for the default column
			columns: RwLock::new(cols.unwrap_or_else(|| Columns::new(columns + 1))),
		});

		persist_db(db.clone());

		db
	}

	fn to_db_column(col: Option<u32>) -> usize {
		col.map_or(0, |c| (c + 1) as usize)
	}
}

impl KeyValueDB for Database {
	fn get(&self, col: Option<u32>, key: &[u8]) -> io::Result<Option<DBValue>> {
		let column = Self::to_db_column(col);
		let columns = self.columns.read().expect_throw(RWLOCK_NO_POISONING_PROOF);
		match columns.columns.get(column) {
			None => Err(io::Error::new(io::ErrorKind::Other, format!("No such column family: {:?}", col))),
			Some(map) => Ok(map.get(key).cloned().map(|v| v.inner).map(DBValue::from_vec)),
		}
	}

	fn get_by_prefix(&self, col: Option<u32>, prefix: &[u8]) -> Option<Box<[u8]>> {
		let col = Self::to_db_column(col);
		let columns = self.columns.read().expect_throw(RWLOCK_NO_POISONING_PROOF);
		match columns.columns.get(col) {
			None => None,
			Some(map) =>
				map.iter().find(|&(ref k ,_)| k.starts_with(prefix))
					.map(|(_, v)| v.inner.clone().into_boxed_slice())
		}
	}

	fn write_buffered(&self, transaction: DBTransaction) {
		let ops = transaction.ops;
		let mut columns = self.columns.write().expect_throw(RWLOCK_NO_POISONING_PROOF);
		for op in ops {
			match op {
				DBOp::Insert { col, key, value } => {
					let col = Self::to_db_column(col);
					if let Some(col) = columns.columns.get_mut(col) {
						col.insert(
							key.into_vec().into(),
							value.into_vec().into(),
						);
					}
				},
				DBOp::Delete { col, key } => {
					let col = Self::to_db_column(col);
					if let Some(col) = columns.columns.get_mut(col) {
						col.remove(&*key);
					}
				},
			}
		}
	}

	fn flush(&self) -> io::Result<()> {
		Ok(())
	}

	// NOTE: clones the whole db
	fn iter<'a>(&'a self, col: Option<u32>) -> Box<Iterator<Item=(Box<[u8]>, Box<[u8]>)> + 'a> {
		let col = Self::to_db_column(col);
		let columns = self.columns.read().expect_throw(RWLOCK_NO_POISONING_PROOF);
		match columns.columns.get(col) {
			Some(map) => Box::new(
				map.clone()
					.into_iter()
					.map(|(k, v)| (k.inner.into_boxed_slice(), v.inner.into_boxed_slice()))
			),
			None => Box::new(None.into_iter()),
		}
	}

	// NOTE: clones the whole db
	fn iter_from_prefix<'a>(&'a self, col: Option<u32>, prefix: &'a [u8])
		-> Box<Iterator<Item=(Box<[u8]>, Box<[u8]>)> + 'a>
	{
		let col = Self::to_db_column(col);
		let columns = self.columns.read().expect_throw(RWLOCK_NO_POISONING_PROOF);
		match columns.columns.get(col) {
			Some(map) => Box::new(
				map.clone()
					.into_iter()
					.skip_while(move |&(ref k, _)| !k.starts_with(prefix))
					.map(|(k, v)| (k.inner.into_boxed_slice(), v.inner.into_boxed_slice()))
			),
			None => Box::new(None.into_iter()),
		}
	}

	// NOTE: not supported
	fn restore(&self, _new_db: &str) -> std::io::Result<()> {
		Err(io::Error::new(io::ErrorKind::Other, "Attempted to restore an in-browser database"))
	}
}


fn persist_db(db: Arc<Database>) {
	const PERSIST_DB_TIMER_MS: i32 = 2_000;

	let window = web_sys::window().expect_throw("are we in a browser?");
	let local_storage = window.local_storage()
		.expect_throw("localStorage should be supported in your browser")
		.unwrap_throw();

	let a = Closure::wrap(Box::new(move || do_persist_db(&db.clone(), &local_storage)) as Box<dyn Fn()>);
	if let Err(e) = window
		.set_interval_with_callback_and_timeout_and_arguments_0(a.as_ref().unchecked_ref(), PERSIST_DB_TIMER_MS)
	{
		console_warn!("Error setting up a timer for localStorage: {:?}", e);
	};
	a.forget();
}


// Won't persist the data if there is write in progress
// (uses try_read() on RwLock).
// Also, it blocks the main thread and is highly inefficient.
fn do_persist_db(db: &Database, local_storage: &Storage) {
	if let Ok(snapshot) = db.columns.try_read() {
		let db_serialized = serde_json::to_string(&*snapshot);
		let db_serialized = match db_serialized {
			Ok(db) => db,
			Err(e) => {
				console_warn!("Error serializing db to json string: {}", e);
				return;
			}
		};
		// We should multiply this by 2 for UTF-16.
		console_log!("Serializing db with size: {}", db_serialized.len());
		if let Err(e) = local_storage.set_item(db.name.as_str(), db_serialized.as_str()) {
			console_warn!("Persisting to local storage has failed: {:?}", e);
		}
	}
}
