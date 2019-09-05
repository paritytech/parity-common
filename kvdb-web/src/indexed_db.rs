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

//! Utility functions to interact with IndexedDB browser API.

use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt, closure::Closure};
use web_sys::{
	IdbDatabase, IdbRequest, IdbOpenDbRequest,
	Event, IdbCursorWithValue,
	IdbTransactionMode,
};
use js_sys::{Array, Uint8Array, ArrayBuffer};

use futures::channel;
use futures::prelude::*;

use kvdb::{DBOp, DBTransaction};

use std::ops::Deref;
use log::{debug, warn};


use crate::Column;


/// Opens the IndexedDB with the given name, version and the specified number of columns
/// (including the default one).
pub fn open(name: &str, version: u32, columns: u32) -> impl Future<Output = IdbDatabase> {
	let (tx, rx) = channel::oneshot::channel::<IdbDatabase>();
	// TODO: handle errors more gracefully,
	// return a Result instead of expect_throw?
	let window = web_sys::window().expect_throw("are we in a browser?");
	let indexed_db = window.indexed_db()
		.expect_throw("IndexDB should be supported in your browser")
		.expect_throw("IndexDB should be supported in your browser");

	let open_request = indexed_db.open_with_u32(name, version)
		.expect_throw("Should be able to open IndexDB");

	try_create_object_stores(&open_request, columns);

	let on_success = Closure::once(move |event: &Event| {
		// Extract database handle from the event
		let target = event.target().expect_throw("Event should have a target");
		let req = target.dyn_ref::<IdbRequest>().expect_throw("Event target is IdbRequest");

		let result = req
			.result()
			.expect_throw("IndexedDB.onsuccess should have a valid result");
		assert!(result.is_instance_of::<IdbDatabase>());

		// errors if the receiving end was dropped before this call
		let _ = tx.send(IdbDatabase::from(result));
	});
	open_request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
	on_success.forget();

	rx.then(|r| future::ready(r.expect("Sender isn't dropped; qed")))
}

fn store_name(num: u32) -> String {
	format!("col{}", num)
}

fn column_to_number(column: Column) -> u32 {
	column.map(|c| c + 1).unwrap_or_default()
}


// Returns js objects representing store names for each column
fn store_names_js(columns: u32) -> Array {
	let column_names = (0..=columns).map(store_name);

	let js_array = Array::new();
	for name in column_names {
		js_array.push(&JsValue::from(name));
	}

	js_array
}

fn try_create_object_stores(req: &IdbOpenDbRequest, columns: u32) {
	let on_upgradeneeded = Closure::once(move |event: &Event| {
		debug!("Upgrading or creating the database");
		// Extract database handle from the event
		let target = event.target().expect_throw("Event should have a target");
		let req = target.dyn_ref::<IdbRequest>().expect_throw("Event target is IdbRequest");
		let result = req.result().expect_throw("IdbRequest should have a result");
		let db: &IdbDatabase = result.unchecked_ref();

		let previous_columns = db.object_store_names().length();

		for name in (previous_columns..=columns).map(store_name) {
			let res = db.create_object_store(name.as_str());
			if let Err(err) = res {
				debug!("error creating object store {}: {:?}", name, err);
			}
		}
	});

	req.set_onupgradeneeded(Some(on_upgradeneeded.as_ref().unchecked_ref()));
	on_upgradeneeded.forget();
}

/// Commit a transaction to the IndexedDB.
pub fn idb_commit_transaction(
	idb: &IdbDatabase,
	txn: &DBTransaction,
	columns: u32,
) -> impl Future<Output = ()> {
	let store_names_js = store_names_js(columns);

	// Create a transaction
	let mode = IdbTransactionMode::Readwrite;
	let idb_txn = idb.transaction_with_str_sequence_and_mode(&store_names_js, mode)
		.expect_throw("Failed to create an IndexedDB transaction");

	// Open object stores (columns)
	let object_stores = (0..=columns).map(|n| {
		idb_txn.object_store(store_name(n).as_str())
			.expect("Object stores were created in try_create_object_stores; qed")
	}).collect::<Vec<_>>();

	for op in &txn.ops {
		match op {
			DBOp::Insert { col, key, value } => {
				let column = column_to_number(*col) as usize;

				// Convert rust bytes to js arrays
				let key_js = Uint8Array::from(key.as_ref());
				let val_js = Uint8Array::from(value.as_ref());

				// Insert key/value pair into the object store
				let res = object_stores[column].put_with_key(val_js.as_ref(), key_js.as_ref());
				if let Err(err) = res {
					warn!("error inserting key/values into col_{}: {:?}", column, err);
				}
			},
			DBOp::Delete { col, key } => {
				let column = column_to_number(*col) as usize;

				// Convert rust bytes to js arrays
				let key_js = Uint8Array::from(key.as_ref());

				// Delete key/value pair from the object store
				let res = object_stores[column].delete(key_js.as_ref());
				if let Err(err) = res {
					warn!("error deleting key from col_{}: {:?}", column, err);
				}
			},
		}
	}

	let (tx, rx) = channel::oneshot::channel::<()>();

	let on_complete = Closure::once(move || {
		let _ = tx.send(());
	});
	idb_txn.set_oncomplete(Some(on_complete.as_ref().unchecked_ref()));
	on_complete.forget();
	// TODO: handle idb_txn.onerror

	rx.map(|_| ())
}


/// Returns a cursor to a database column with the given column number.
pub fn idb_cursor(idb: &IdbDatabase, col: u32) -> impl Stream<Item = (Vec<u8>, Vec<u8>)> {
	// TODO: we could read all the columns in one db transaction
	let store_name = store_name(col);
	let store_name = store_name.as_str();
	let txn = idb.transaction_with_str(store_name)
		.expect_throw("Failed to create an IndexedDB transaction");

	let store = txn.object_store(store_name).expect_throw("Opening a store shouldn't fail");
	let cursor = store.open_cursor().expect_throw("Opening a cursor shoudn't fail");

	let (tx, rx) = channel::mpsc::unbounded();

	let on_cursor = Closure::wrap(Box::new(move |event: &Event| {
		// Extract the cursor from the event
		let target = event.target().expect_throw("on_cursor should have a target");
		let req = target.dyn_ref::<IdbRequest>().expect_throw("target should be IdbRequest");
		let result = req.result().expect_throw("IdbRequest should have a result");
		let cursor: &IdbCursorWithValue = result.unchecked_ref();

		if let (Ok(key), Ok(value)) = (cursor.deref().key(), cursor.value()) {
			let k: &ArrayBuffer = key.unchecked_ref();
			let v: &Uint8Array = value.unchecked_ref();

			// Copy js arrays into rust `Vec`s
			let mut kv = vec![0u8; k.byte_length() as usize];
			let mut vv = vec![0u8; v.byte_length() as usize];
			Uint8Array::new(k).copy_to(&mut kv[..]);
			v.copy_to(&mut vv[..]);

			if let Err(e) = tx.unbounded_send((kv, vv)) {
				warn!("on_cursor: error sending to a channel {:?}", e);
			}
			if let Err(e) = cursor.deref().continue_() {
				warn!("cursor advancement has failed {:?}", e);
			}
		} else {
			// we're done
			tx.close_channel();
		}
	}) as Box<dyn FnMut(&Event)>);

	cursor.set_onsuccess(Some(on_cursor.as_ref().unchecked_ref()));
	on_cursor.forget();

	rx
}
