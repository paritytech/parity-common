// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use kvdb::{DBOp, DBTransaction, DBValue, KeyValueDB};
use parity_util_mem::MallocSizeOf;
use parking_lot::RwLock;
use std::{
	collections::{BTreeMap, HashMap},
	io,
};

/// A key-value database fulfilling the `KeyValueDB` trait, living in memory.
/// This is generally intended for tests and is not particularly optimized.
#[derive(Default, MallocSizeOf)]
pub struct InMemory {
	columns: RwLock<HashMap<u32, BTreeMap<Vec<u8>, DBValue>>>,
}

/// Create an in-memory database with the given number of columns.
/// Columns will be indexable by 0..`num_cols`
pub fn create(num_cols: u32) -> InMemory {
	let mut cols = HashMap::new();

	for idx in 0..num_cols {
		cols.insert(idx, BTreeMap::new());
	}

	InMemory { columns: RwLock::new(cols) }
}

impl KeyValueDB for InMemory {
	fn get(&self, col: u32, key: &[u8]) -> io::Result<Option<DBValue>> {
		let columns = self.columns.read();
		match columns.get(&col) {
			None => Err(io::Error::new(io::ErrorKind::Other, format!("No such column family: {:?}", col))),
			Some(map) => Ok(map.get(key).cloned()),
		}
	}

	fn get_by_prefix(&self, col: u32, prefix: &[u8]) -> Option<Box<[u8]>> {
		let columns = self.columns.read();
		match columns.get(&col) {
			None => None,
			Some(map) => {
				map.iter().find(|&(ref k, _)| k.starts_with(prefix)).map(|(_, v)| v.to_vec().into_boxed_slice())
			}
		}
	}

	fn write_buffered(&self, transaction: DBTransaction) {
		let mut columns = self.columns.write();
		let ops = transaction.ops;
		for op in ops {
			match op {
				DBOp::Insert { col, key, value } => {
					if let Some(col) = columns.get_mut(&col) {
						col.insert(key.into_vec(), value);
					}
				}
				DBOp::Delete { col, key } => {
					if let Some(col) = columns.get_mut(&col) {
						col.remove(&*key);
					}
				}
			}
		}
	}

	fn flush(&self) -> io::Result<()> {
		Ok(())
	}

	fn iter<'a>(&'a self, col: u32) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a> {
		match self.columns.read().get(&col) {
			Some(map) => Box::new(
				// TODO: worth optimizing at all?
				map.clone().into_iter().map(|(k, v)| (k.into_boxed_slice(), v.into_boxed_slice())),
			),
			None => Box::new(None.into_iter()),
		}
	}

	fn iter_from_prefix<'a>(
		&'a self,
		col: u32,
		prefix: &'a [u8],
	) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a> {
		match self.columns.read().get(&col) {
			Some(map) => Box::new(
				map.clone()
					.into_iter()
					.filter(move |&(ref k, _)| k.starts_with(prefix))
					.map(|(k, v)| (k.into_boxed_slice(), v.into_boxed_slice())),
			),
			None => Box::new(None.into_iter()),
		}
	}

	fn restore(&self, _new_db: &str) -> io::Result<()> {
		Err(io::Error::new(io::ErrorKind::Other, "Attempted to restore in-memory database"))
	}
}

#[cfg(test)]
mod tests {
	use super::create;
	use kvdb_shared_tests as st;
	use std::io;

	#[test]
	fn get_fails_with_non_existing_column() -> io::Result<()> {
		let db = create(1);
		st::test_get_fails_with_non_existing_column(&db)
	}

	#[test]
	fn put_and_get() -> io::Result<()> {
		let db = create(1);
		st::test_put_and_get(&db)
	}

	#[test]
	fn delete_and_get() -> io::Result<()> {
		let db = create(1);
		st::test_delete_and_get(&db)
	}

	#[test]
	fn iter() -> io::Result<()> {
		let db = create(1);
		st::test_iter(&db)
	}

	#[test]
	fn iter_from_prefix() -> io::Result<()> {
		let db = create(1);
		st::test_iter_from_prefix(&db)
	}

	#[test]
	fn complex() -> io::Result<()> {
		let db = create(1);
		st::test_complex(&db)
	}
}
