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

//! KeyValueDB implementation for sled database.

use kvdb::{KeyValueDB, DBTransaction, DBValue, DBOp};
use std::io;
use sled::Transactional as _;
use log::warn;

const KB: u64 = 1024;
const MB: u64 = 1024 * KB;
const DB_DEFAULT_MEMORY_BUDGET_MB: u64 = 128;

fn other_io_err<E>(e: E) -> io::Error where E: Into<Box<dyn std::error::Error + Send + Sync>> {
	io::Error::new(io::ErrorKind::Other, e)
}

pub struct Database {
	db: sled::Db,
	// `sled::Tree` corresponds to a `Column` in the KeyValueDB terminology.
	columns: Vec<sled::Tree>,
	path: String,
}

// TODO: docs
#[derive(Default)]
pub struct DatabaseConfig {
	pub columns: Option<u8>,
	pub memory_budget_mb: Option<u64>,
}

impl DatabaseConfig {
	pub fn with_columns(columns: u8) -> Self {
		Self {
			columns: Some(columns),
			memory_budget_mb: None,
		}
	}
	pub fn memory_budget(&self) -> u64 {
		self.memory_budget_mb.unwrap_or(DB_DEFAULT_MEMORY_BUDGET_MB) * MB
	}
}

fn to_sled_config(config: &DatabaseConfig, path: &str) -> sled::Config {
	let conf = sled::Config::default()
		.path(path)
		.cache_capacity(config.memory_budget() / 2)
		.flush_every_ms(Some(2_000)); // TODO: a random constant
	// .snapshot_after_ops(100_000);
	conf
}

fn col_name(col: u8) -> String {
	format!("col{}", col)
}

impl Database {
	pub fn open(config: &DatabaseConfig, path: &str) -> sled::Result<Database> {
		let conf = to_sled_config(config, path);

		let db = conf.open()?;
		let num_columns = config.columns.map_or(1, |c| c + 1);
		let columns = (0..num_columns)
			.map(|i| db.open_tree(col_name(i).as_bytes()))
			.collect::<sled::Result<Vec<_>>>()?;

		Ok(Database {
			db,
			columns,
			path: path.to_string(),
		})
	}

	/// The number of non-default column families.
	pub fn num_columns(&self) -> u8 {
		self.columns.len() as u8 - 1
	}

	/// Drop a column family.
	pub fn drop_column(&mut self) -> io::Result<()> {
		if let Some(col) = self.columns.pop() {
			let name = col_name(self.columns.len() as u8);
			drop(col);
			self.db.drop_tree(name.as_bytes()).map_err(other_io_err)?;
		}
		Ok(())
	}

	/// Add a column family.
	pub fn add_column(&mut self) -> io::Result<()> {
		let col = self.columns.len() as u8;
		let name = col_name(col);
		let tree = self.db.open_tree(name.as_bytes()).map_err(other_io_err)?;
		self.columns.push(tree);
		Ok(())
	}

	fn to_sled_column(col: Option<u32>) -> u8 {
		col.map_or(0, |c| (c + 1) as u8)
	}
}

impl KeyValueDB for Database {
	fn get(&self, col: Option<u32>, key: &[u8]) -> io::Result<Option<DBValue>> {
		let col = Self::to_sled_column(col);
		self.columns[col as usize]
			.get(key)
			.map(|maybe| maybe.map(|ivec| DBValue::from_slice(ivec.as_ref())))
			.map_err(other_io_err)
	}

	fn get_by_prefix(&self, col: Option<u32>, prefix: &[u8]) -> Option<Box<[u8]>> {
		self.iter_from_prefix(col, prefix).next().map(|(_, v)| v)
		// TODO: an optimized version below works only
		// in the case of prefix.len() < key.len()
		//
		// let col = Self::to_sled_column(col);
		// self.columns[col as usize]
		// 	.get_gt(prefix)
		// 	.ok() // ignore errors
		// 	.and_then(|maybe| maybe.and_then(|(k, v)| {
		// 		if k.as_ref().starts_with(prefix) {
		// 			Some(Box::from(v.as_ref()))
		// 		} else {
		// 			None
		// 		}
		// 	}))
	}

	fn write_buffered(&self, tr: DBTransaction) {
		// REVIEW: not sure if it's semantically correct
		//         to apply an ACID transaction here
		let result = self.write(tr);
		if result.is_err() {
			warn!(target: "kvdb-sled", "transaction has failed")
		}
	}

	fn write(&self, tr: DBTransaction) -> io::Result<()> {
		// FIXME: sled currently support transactions only on tuples of trees,
		// see https://github.com/spacejam/sled/issues/382#issuecomment-526548082
		// TODO: implement for more sizes via macro
		let result = match &self.columns[..] {
			[c1] => c1.transaction(|c1| {
				let columns = [c1];
				for op in &tr.ops {
					match op {
						DBOp::Insert { col, key, value } => {
							let col = Self::to_sled_column(*col);
							columns[col as usize].insert(key.as_ref(), value.as_ref())?;
						},
						DBOp::Delete { col, key } => {
							let col = Self::to_sled_column(*col);
							columns[col as usize].remove(key.as_ref())?;
						}
					}
				}
				Ok(())
			}),
			[c1, c2, c3, c4, c5, c6, c7, c8, c9] => {
				(c1, c2, c3, c4, c5, c6, c7, c8, c9).transaction(|(c1, c2, c3, c4, c5, c6, c7, c8, c9)| {
					let columns = [c1, c2, c3, c4, c5, c6, c7, c8, c9];
					for op in &tr.ops {
						match op {
							DBOp::Insert { col, key, value } => {
								let col = Self::to_sled_column(*col);
								columns[col as usize].insert(key.as_ref(), value.as_ref())?;
							},
							DBOp::Delete { col, key } => {
								let col = Self::to_sled_column(*col);
								columns[col as usize].remove(key.as_ref())?;
							}
						}
					}
					Ok(())
				})
			},
			_ => panic!("only up to 9 columns are supported ATM"),
		};
		result.map_err(|_| other_io_err("transaction has failed"))
	}

	fn flush(&self) -> io::Result<()> {
		for tree in &self.columns {
			tree.flush().map_err(other_io_err)?;
		}
		Ok(())
	}

	fn iter<'a>(&'a self, col: Option<u32>) -> Box<dyn Iterator<Item=(Box<[u8]>, Box<[u8]>)> + 'a> {
		let col = Self::to_sled_column(col);
		let iter = DatabaseIter {
			inner: self.columns[col as usize].iter(),
		};
		Box::new(iter.into_iter())
	}

	fn iter_from_prefix<'a>(&'a self, col: Option<u32>, prefix: &'a [u8])
		-> Box<dyn Iterator<Item=(Box<[u8]>, Box<[u8]>)> + 'a>
	{
		let col = Self::to_sled_column(col);
		let iter = DatabaseIter {
			inner: self.columns[col as usize].scan_prefix(prefix),
		};
		Box::new(iter.into_iter())
	}

	fn restore(&self, new_db: &str) -> io::Result<()> {
		unimplemented!("TODO")
	}
}

struct DatabaseIter {
	inner: sled::Iter,
}

impl std::iter::Iterator for DatabaseIter {
	type Item = (Box<[u8]>, Box<[u8]>);
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next().and_then(|result| {
			let (k, v) = result.ok()?; // ignore the error
			Some((Box::from(k.as_ref()), Box::from(v.as_ref())))
		})
	}
}

impl Drop for Database {
	fn drop(&mut self) {
		// write all buffered changes if we can.
		let _ = self.flush();
	}
}


#[cfg(test)]
mod tests {
	use tempdir::TempDir;
	use super::*;

	#[test]
	fn add_columns() {
		let tempdir = TempDir::new("sled-test-add_columns").unwrap().path().to_str().unwrap().to_owned();

		// open empty, add 5.
		{
			let config = DatabaseConfig::default();
			let mut db = Database::open(&config, &tempdir).unwrap();
			assert_eq!(db.num_columns(), 0);

			for i in 0..5 {
				db.add_column().unwrap();
				assert_eq!(db.num_columns(), i + 1);
			}
		}

		// reopen as 5.
		{
			let config_5 = DatabaseConfig::with_columns(5);
			let db = Database::open(&config_5, &tempdir).unwrap();
			assert_eq!(db.num_columns(), 5);
		}
	}

	#[test]
	fn drop_columns() {
		let tempdir = TempDir::new("sled-test-drop_columns").unwrap().path().to_str().unwrap().to_owned();

		// open 5, remove all.
		{
			let config_5 = DatabaseConfig::with_columns(5);
			let mut db = Database::open(&config_5, &tempdir).unwrap();
			assert_eq!(db.num_columns(), 5);

			for i in (0..5).rev() {
				db.drop_column().unwrap();
				assert_eq!(db.num_columns(), i);
			}
		}

		// reopen as 0.
		{
			let config = DatabaseConfig::default();
			let db = Database::open(&config, &tempdir).unwrap();
			assert_eq!(db.num_columns(), 0);
		}
	}

	#[test]
	fn write_clears_buffered_ops() {
		let tempdir = TempDir::new("sled-test-write_clears_buffered_ops").unwrap().path().to_str().unwrap().to_owned();
		let config = DatabaseConfig::default();
		let db = Database::open(&config, &tempdir).unwrap();

		let mut batch = db.transaction();
		batch.put(None, b"foo", b"bar");
		db.write_buffered(batch);

		let mut batch = db.transaction();
		batch.put(None, b"foo", b"baz");
		db.write(batch).unwrap();

		assert_eq!(db.get(None, b"foo").unwrap().unwrap().as_ref(), b"baz");
	}

	#[test]
	fn test_db() {
		let tempdir = TempDir::new("sled-test-write_clears_buffered_ops").unwrap().path().to_str().unwrap().to_owned();
		let config = DatabaseConfig::default();
		let db = Database::open(&config, &tempdir).unwrap();
		let key1 = b"02c69be41d0b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc";
		let key2 = b"03c69be41d0b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc";
		let key3 = b"01c69be41d0b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc";

		let mut batch = db.transaction();
		batch.put(None, key1, b"cat");
		batch.put(None, key2, b"dog");
		db.write(batch).unwrap();

		assert_eq!(&*db.get(None, key1).unwrap().unwrap(), b"cat");

		let contents: Vec<_> = db.iter(None).into_iter().collect();
		assert_eq!(contents.len(), 2);
		assert!(contents[0].0.to_vec() == key1.to_vec());
		assert_eq!(&*contents[0].1, b"cat");
		assert_eq!(contents[1].0.to_vec(), key2.to_vec());
		assert_eq!(&*contents[1].1, b"dog");

		let mut batch = db.transaction();
		batch.delete(None, key1);
		db.write(batch).unwrap();

		assert!(db.get(None, key1).unwrap().is_none());

		let mut batch = db.transaction();
		batch.put(None, key1, b"cat");
		db.write(batch).unwrap();

		let mut transaction = db.transaction();
		transaction.put(None, key3, b"elephant");
		transaction.delete(None, key1);
		db.write(transaction).unwrap();
		assert!(db.get(None, key1).unwrap().is_none());
		assert_eq!(&*db.get(None, key3).unwrap().unwrap(), b"elephant");

		assert_eq!(&*db.get_by_prefix(None, key3).unwrap(), b"elephant");
		assert_eq!(&*db.get_by_prefix(None, key2).unwrap(), b"dog");

		let mut transaction = db.transaction();
		transaction.put(None, key1, b"horse");
		transaction.delete(None, key3);
		db.write_buffered(transaction);
		assert!(db.get(None, key3).unwrap().is_none());
		assert_eq!(&*db.get(None, key1).unwrap().unwrap(), b"horse");

		db.flush().unwrap();
		assert!(db.get(None, key3).unwrap().is_none());
		assert_eq!(&*db.get(None, key1).unwrap().unwrap(), b"horse");
	}
}
