// Copyright 2026 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! End-to-end checks for `ColumnConfig::path`: SST files must land in the
//! override directory, survive reopen, and a reopen without the override must
//! not silently serve an empty column.

use kvdb::KeyValueDB;
use kvdb_rocksdb::{Database, DatabaseConfig};
use std::{fs, path::Path};

const COLD_COL: u32 = 1;
const NUM_KEYS: u32 = 512;
const VALUE_LEN: usize = 8 * 1024; // 512 * 8 KiB = 4 MiB, enough to force SST flushes at a 1 MiB budget

fn sst_files(dir: &Path) -> Vec<String> {
	let mut out = Vec::new();
	let mut stack = vec![dir.to_path_buf()];
	while let Some(d) = stack.pop() {
		if let Ok(entries) = fs::read_dir(&d) {
			for e in entries.flatten() {
				let p = e.path();
				if p.is_dir() {
					stack.push(p);
				} else if p.extension().map_or(false, |x| x == "sst") {
					out.push(p.file_name().unwrap().to_string_lossy().into_owned());
				}
			}
		}
	}
	out
}

fn config(cold_path: Option<&Path>) -> DatabaseConfig {
	let mut cfg = DatabaseConfig::with_columns(2);
	cfg.columns[COLD_COL as usize].memory_budget = Some(1); // tiny budget so 4 MiB of writes flushes to SSTs
	cfg.columns[COLD_COL as usize].path = cold_path.map(|p| p.to_path_buf());
	cfg
}

fn value(i: u32) -> Vec<u8> {
	let mut v = vec![0u8; VALUE_LEN];
	v[..4].copy_from_slice(&i.to_le_bytes());
	v
}

fn write_all(db: &Database) {
	for i in 0..NUM_KEYS {
		let mut tx = db.transaction();
		tx.put(COLD_COL, &i.to_le_bytes(), &value(i));
		if i == 0 {
			tx.put(0, b"main", b"stays-on-main-path");
		}
		db.write(tx).unwrap();
	}
}

fn read_all(db: &Database) {
	for i in 0..NUM_KEYS {
		let got = db.get(COLD_COL, &i.to_le_bytes()).unwrap();
		assert_eq!(got.as_deref(), Some(value(i).as_slice()), "key {} lost", i);
	}
	assert_eq!(db.get(0, b"main").unwrap().as_deref(), Some(&b"stays-on-main-path"[..]));
}

#[test]
fn cold_column_ssts_land_on_override_path_and_survive_reopen() {
	let main = tempfile::tempdir().unwrap();
	let cold = tempfile::tempdir().unwrap();
	let cold_dir = cold.path().join("col1");

	let db = Database::open(&config(Some(&cold_dir)), main.path()).unwrap();
	write_all(&db);
	read_all(&db);
	drop(db);

	let cold_ssts = sst_files(&cold_dir);
	assert!(!cold_ssts.is_empty(), "no SST files under the override dir {:?}", cold_dir);
	let main_ssts = sst_files(main.path());
	for f in &cold_ssts {
		assert!(!main_ssts.contains(f), "SST {} present in BOTH main and override dirs", f);
	}

	let db = Database::open(&config(Some(&cold_dir)), main.path()).unwrap();
	read_all(&db);
}

#[test]
fn reopen_without_override_does_not_silently_serve_data() {
	let main = tempfile::tempdir().unwrap();
	let cold = tempfile::tempdir().unwrap();
	let cold_dir = cold.path().join("col1");

	let db = Database::open(&config(Some(&cold_dir)), main.path()).unwrap();
	write_all(&db);
	drop(db);
	assert!(!sst_files(&cold_dir).is_empty());

	// Simulates a node restarted with the flag dropped (or the volume unmounted): RocksDB must
	// refuse to open rather than come up with a hole where the cold column was.
	match Database::open(&config(None), main.path()) {
		Err(e) => {
			eprintln!("reopen without override failed loudly (good): {}", e);
		},
		Ok(db) => {
			let sample = db.get(COLD_COL, &0u32.to_le_bytes()).unwrap();
			assert!(
				sample.is_none(),
				"reopen without override silently served cold-column data; placement suspect"
			);
			panic!("reopen without override succeeded with an empty cold column: silent data hole");
		},
	}
}
