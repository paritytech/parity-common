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
use sled::{Tree, Db, Iter};
use std::io;

const KB: u64 = 1024;
const MB: u64 = 1024 * KB;
const DB_DEFAULT_MEMORY_BUDGET_MB: u64 = 128;

fn other_io_err<E>(e: E) -> io::Error where E: Into<Box<dyn std::error::Error + Send + Sync>> {
	io::Error::new(io::ErrorKind::Other, e)
}

pub struct Database {
	// sled currently support transactions only on tuples of trees (up to 10),
	// not vecs because it might make the trees typed in the future.
	// see https://github.com/spacejam/sled/issues/382#issuecomment-526548082
	// sled `Tree` corresponds to a `Column` in the KeyValueDB terminology.
	columns: Vec<Tree>,
	path: String,
	num_columns: u8,
}

// TODO: docs
pub struct DatabaseConfig {
	pub columns: Option<u8>,
	pub memory_budget_mb: Option<u64>,
	pub path: String,
}

impl DatabaseConfig {
	pub fn memory_budget(&self) -> u64 {
		self.memory_budget_mb.unwrap_or(DB_DEFAULT_MEMORY_BUDGET_MB) * MB
	}
}

impl Database {
	pub fn open(config: DatabaseConfig) -> sled::Result<Database> {
		let conf = sled::Config::default()
			.path(&config.path)
			.cache_capacity(config.memory_budget() / 2)
			.flush_every_ms(Some(1_000));
			// .snapshot_after_ops(100_000);

		let db = conf.open()?;
		let num_columns = config.columns.map_or(0, |c| c + 1);
		let columns = (0..=num_columns)
			.map(|i| db.open_tree(format!("col{}", i).as_bytes()))
			.collect::<sled::Result<Vec<_>>>()?;

		Ok(Database {
			columns,
			path: config.path,
			num_columns,
		})
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
		let col = Self::to_sled_column(col);
		self.columns[col as usize]
			.get_gt(prefix)
			.ok() // ignore errors
			.and_then(|maybe| maybe.and_then(|(k, v)| {
				if k.as_ref().starts_with(prefix) {
					Some(Box::from(v.as_ref()))
				} else {
					None
				}
			}))
	}

	fn write_buffered(&self, transaction: DBTransaction) {
		Database::write_buffered(self, transaction)
	}

	fn write(&self, transaction: DBTransaction) -> io::Result<()> {
		Database::write(self, transaction)
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
	inner: Iter,
}

impl std::iter::Iterator for DatabaseIter {
	type Item = (Box<[u8]>, Box<[u8]>);
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next().and_then(|result| {
			let (k, v) = result.ok()?;
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
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
