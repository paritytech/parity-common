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
const DB_DEFAULT_MEMORY_BUDGET_MB: u64 = 1024;

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
	pub columns: u32,
	pub memory_budget_mb: Option<u64>,
}

impl DatabaseConfig {
	pub fn with_columns(columns: u32) -> Self {
		Self {
			columns,
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
		.cache_capacity(config.memory_budget())
		.flush_every_ms(Some(2_000)); // TODO: a random constant
	// .snapshot_after_ops(100_000);
	conf
}

fn col_name(col: u32) -> String {
	format!("col{}", col)
}

impl Database {
	pub fn open(config: &DatabaseConfig, path: &str) -> sled::Result<Database> {
		let conf = to_sled_config(config, path);

		let db = conf.open()?;
		let num_columns = config.columns;
		let columns = (0..num_columns)
			.map(|i| db.open_tree(col_name(i).as_bytes()))
			.collect::<sled::Result<Vec<_>>>()?;

		Ok(Database {
			db,
			columns,
			path: path.to_string(),
		})
	}

	/// The database path.
	pub fn path(&self) -> &str {
		&self.path
	}

	/// The number of column families.
	pub fn num_columns(&self) -> u32 {
		self.columns.len() as u32
	}

	/// Drop a column family.
	pub fn drop_column(&mut self) -> io::Result<()> {
		if let Some(col) = self.columns.pop() {
			let name = col_name(self.num_columns());
			drop(col);
			self.db.drop_tree(name.as_bytes()).map_err(other_io_err)?;
		}
		Ok(())
	}

	/// Add a column family.
	pub fn add_column(&mut self) -> io::Result<()> {
		let col = self.num_columns();
		let name = col_name(col);
		let tree = self.db.open_tree(name.as_bytes()).map_err(other_io_err)?;
		self.columns.push(tree);
		Ok(())
	}
}

impl parity_util_mem::MallocSizeOf for Database {
	fn size_of(&self, _ops: &mut parity_util_mem::MallocSizeOfOps) -> usize {
		// TODO
		(DB_DEFAULT_MEMORY_BUDGET_MB * MB) as usize
	}
}

impl KeyValueDB for Database {
	fn get(&self, col: u32, key: &[u8]) -> io::Result<Option<DBValue>> {
		self.columns[col as usize]
			.get(key)
			.map(|maybe| maybe.map(|ivec| ivec.to_vec()))
			.map_err(other_io_err)
	}

	fn get_by_prefix(&self, col: u32, prefix: &[u8]) -> Option<Box<[u8]>> {
		self.iter_from_prefix(col, prefix).next().map(|(_, v)| v)
	}

	fn write_buffered(&self, tr: DBTransaction) {
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
							let val = AsRef::<[u8]>::as_ref(&value);
							columns[*col as usize].insert(key.as_ref(), val)?;
						},
						DBOp::Delete { col, key } => {
							columns[*col as usize].remove(key.as_ref())?;
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
								let val = AsRef::<[u8]>::as_ref(&value);
								columns[*col as usize].insert(key.as_ref(), val)?;
							},
							DBOp::Delete { col, key } => {
								columns[*col as usize].remove(key.as_ref())?;
							}
						}
					}
					Ok(())
				})
			},
			_ => panic!("only up to 9 columns are supported ATM, given {}", self.columns.len()),
		};
		result.map_err(|_| other_io_err("transaction has failed"))
	}

	fn flush(&self) -> io::Result<()> {
		for tree in &self.columns {
			tree.flush().map_err(other_io_err)?;
		}
		Ok(())
	}

	fn iter<'a>(&'a self, col: u32) -> Box<dyn Iterator<Item=(Box<[u8]>, Box<[u8]>)> + 'a> {
		let iter = DatabaseIter {
			inner: self.columns[col as usize].iter(),
		};
		Box::new(iter.into_iter())
	}

	fn iter_from_prefix<'a>(&'a self, col: u32, prefix: &'a [u8])
		-> Box<dyn Iterator<Item=(Box<[u8]>, Box<[u8]>)> + 'a>
	{
		let iter = DatabaseIter {
			inner: self.columns[col as usize].scan_prefix(prefix),
		};
		Box::new(iter.into_iter())
	}

	fn restore(&self, _new_db: &str) -> io::Result<()> {
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
	use super::*;
	use kvdb_shared_tests as st;
	use std::io::{self, Read};
	use tempdir::TempDir;

	fn create(columns: u32) -> io::Result<Database> {
		let tempdir = TempDir::new("")?;
		let config = DatabaseConfig::with_columns(columns);
		Database::open(&config, tempdir.path().to_str().expect("tempdir path is valid unicode"))
	}

	#[test]
	fn get_fails_with_non_existing_column() -> io::Result<()> {
		let db = create(1)?;
		st::test_get_fails_with_non_existing_column(&db)
	}

	#[test]
	fn put_and_get() -> io::Result<()> {
		let db = create(1)?;
		st::test_put_and_get(&db)
	}

	#[test]
	fn delete_and_get() -> io::Result<()> {
		let db = create(1)?;
		st::test_delete_and_get(&db)
	}

	#[test]
	fn iter() -> io::Result<()> {
		let db = create(1)?;
		st::test_iter(&db)
	}

	#[test]
	fn iter_from_prefix() -> io::Result<()> {
		let db = create(1)?;
		st::test_iter_from_prefix(&db)
	}

	#[test]
	fn complex() -> io::Result<()> {
		let db = create(1)?;
		st::test_complex(&db)
	}

	#[test]
	fn stats() -> io::Result<()> {
		let db = create(3)?;
		st::test_io_stats(&db)
	}

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
}
