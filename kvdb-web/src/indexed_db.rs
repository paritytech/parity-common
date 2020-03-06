// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Utility functions to interact with IndexedDB browser API.

use js_sys::{Array, ArrayBuffer, Uint8Array};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, IdbCursorWithValue, IdbDatabase, IdbOpenDbRequest, IdbRequest, IdbTransactionMode};

use futures::channel;
use futures::prelude::*;

use kvdb::{DBOp, DBTransaction};

use log::{debug, warn};
use std::ops::Deref;

use crate::error::Error;

pub struct IndexedDB {
	pub version: u32,
	pub columns: u32,
	pub inner: super::SendWrapper<IdbDatabase>,
}

/// Opens the IndexedDB with the given name, version and the specified number of columns
/// (including the default one).
pub fn open(name: &str, version: Option<u32>, columns: u32) -> impl Future<Output = Result<IndexedDB, Error>> {
	let (tx, rx) = channel::oneshot::channel::<IndexedDB>();

	let window = match web_sys::window() {
		Some(window) => window,
		None => return future::Either::Right(future::err(Error::WindowNotAvailable)),
	};
	let idb_factory = window.indexed_db();

	let idb_factory = match idb_factory {
		Ok(idb_factory) => idb_factory.expect("We can't get a null pointer back; qed"),
		Err(err) => return future::Either::Right(future::err(Error::NotSupported(format!("{:?}", err)))),
	};

	let open_request = match version {
		Some(version) => idb_factory.open_with_u32(name, version).expect("TypeError is not possible with Rust; qed"),
		None => idb_factory.open(name).expect("TypeError is not possible with Rust; qed"),
	};

	try_create_missing_stores(&open_request, columns, version);

	let on_success = Closure::once(move |event: &Event| {
		// Extract database handle from the event
		let target = event.target().expect("Event should have a target; qed");
		let req = target.dyn_ref::<IdbRequest>().expect("Event target is IdbRequest; qed");

		let result = req.result().expect("IndexedDB.onsuccess should have a valid result; qed");
		assert!(result.is_instance_of::<IdbDatabase>());

		let db = IdbDatabase::from(result);
		// JS returns version as f64
		let version = db.version().round() as u32;
		let columns = db.object_store_names().length();

		// errors if the receiving end was dropped before this call
		let _ = tx.send(IndexedDB { version, columns, inner: super::SendWrapper::new(db) });
	});
	open_request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
	on_success.forget();

	future::Either::Left(rx.then(|r| future::ok(r.expect("Sender isn't dropped; qed"))))
}

fn store_name(num: u32) -> String {
	format!("col{}", num)
}

// Returns js objects representing store names for each column
fn store_names_js(columns: u32) -> Array {
	let column_names = (0..columns).map(store_name);

	let js_array = Array::new();
	for name in column_names {
		js_array.push(&JsValue::from(name));
	}

	js_array
}

fn try_create_missing_stores(req: &IdbOpenDbRequest, columns: u32, version: Option<u32>) {
	let on_upgradeneeded = Closure::once(move |event: &Event| {
		debug!("Upgrading or creating the database to version {:?}, columns {}", version, columns);
		// Extract database handle from the event
		let target = event.target().expect("Event should have a target; qed");
		let req = target.dyn_ref::<IdbRequest>().expect("Event target is IdbRequest; qed");
		let result = req.result().expect("IdbRequest should have a result; qed");
		let db: &IdbDatabase = result.unchecked_ref();

		let previous_columns = db.object_store_names().length();
		debug!("Previous version: {}, columns {}", db.version(), previous_columns);

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
pub fn idb_commit_transaction(idb: &IdbDatabase, txn: &DBTransaction, columns: u32) -> impl Future<Output = ()> {
	let store_names_js = store_names_js(columns);

	// Create a transaction
	let mode = IdbTransactionMode::Readwrite;
	let idb_txn = idb
		.transaction_with_str_sequence_and_mode(&store_names_js, mode)
		.expect("The provided mode and store names are valid; qed");

	// Open object stores (columns)
	let object_stores = (0..columns)
		.map(|n| {
			idb_txn
				.object_store(store_name(n).as_str())
				.expect("Object stores were created in try_create_object_stores; qed")
		})
		.collect::<Vec<_>>();

	for op in &txn.ops {
		match op {
			DBOp::Insert { col, key, value } => {
				let column = *col as usize;
				// Convert rust bytes to js arrays
				let key_js = Uint8Array::from(key.as_ref());
				let val_js = Uint8Array::from(value.as_ref());

				// Insert key/value pair into the object store
				let res = object_stores[column].put_with_key(val_js.as_ref(), key_js.as_ref());
				if let Err(err) = res {
					warn!("error inserting key/values into col_{}: {:?}", column, err);
				}
			}
			DBOp::Delete { col, key } => {
				let column = *col as usize;
				// Convert rust bytes to js arrays
				let key_js = Uint8Array::from(key.as_ref());

				// Delete key/value pair from the object store
				let res = object_stores[column].delete(key_js.as_ref());
				if let Err(err) = res {
					warn!("error deleting key from col_{}: {:?}", column, err);
				}
			}
		}
	}

	let (tx, rx) = channel::oneshot::channel::<()>();

	let on_complete = Closure::once(move || {
		let _ = tx.send(());
	});
	idb_txn.set_oncomplete(Some(on_complete.as_ref().unchecked_ref()));
	on_complete.forget();

	let on_error = Closure::once(move || {
		warn!("Failed to commit a transaction to IndexedDB");
	});
	idb_txn.set_onerror(Some(on_error.as_ref().unchecked_ref()));
	on_error.forget();

	rx.map(|_| ())
}

/// Returns a cursor to a database column with the given column number.
pub fn idb_cursor(idb: &IdbDatabase, col: u32) -> impl Stream<Item = (Vec<u8>, Vec<u8>)> {
	// TODO: we could read all the columns in one db transaction
	let store_name = store_name(col);
	let store_name = store_name.as_str();
	let txn = idb.transaction_with_str(store_name).expect("The stores were created on open: {}; qed");

	let store = txn.object_store(store_name).expect("Opening a store shouldn't fail; qed");
	let cursor = store.open_cursor().expect("Opening a cursor shouldn't fail; qed");

	let (tx, rx) = channel::mpsc::unbounded();

	let on_cursor = Closure::wrap(Box::new(move |event: &Event| {
		// Extract the cursor from the event
		let target = event.target().expect("on_cursor should have a target; qed");
		let req = target.dyn_ref::<IdbRequest>().expect("target should be IdbRequest; qed");
		let result = req.result().expect("IdbRequest should have a result; qed");
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
