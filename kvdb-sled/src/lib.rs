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
use sled::{Tree, Db};

const KB: usize = 1024;
const MB: usize = 1024 * KB;
const DB_DEFAULT_MEMORY_BUDGET_MB: usize = 128;

struct Database {
    // sled currently support transactions only on tuples of trees (up to 10), 
    // not vecs because it might make the trees typed in the future.
    // see https://github.com/spacejam/sled/issues/382#issuecomment-526548082
    // sled `Tree` corresponds to a `Column` in the KeyValueDB terminology.
    columns: (Tree, Tree, Tree, Tree, Tree, Tree, Tree, Tree, Tree, Tree),
    path: String,
}

struct DatabaseConfig {
    pub columns: Option<u8>,
    pub memory_budget_mb: Option<usize>,
    pub path: String,
}

impl DatabaseConfig {
    
    pub fn memory_budget(&self) -> usize {
		self.memory_budget.unwrap_or(DB_DEFAULT_MEMORY_BUDGET_MB) * MB
	}

	pub fn memory_budget_per_col(&self) -> usize {
		self.memory_budget() / self.columns.unwrap_or(1) as usize
	}
}

impl Database {
    fn open(config: &DatabaseConfig) -> sled::Result<Database> {
        let _config = sled::Config::default()
            .path(path)
            .cache_capacity(10_000_000_000)
            .flush_every_ms(Some(1000))
            .snapshot_after_ops(100_000);
    }
}

impl KeyValueDB for Database {
	fn get(&self, col: Option<u32>, key: &[u8]) -> io::Result<Option<DBValue>> {
		Database::get(self, col, key)
	}

	fn get_by_prefix(&self, col: Option<u32>, prefix: &[u8]) -> Option<Box<[u8]>> {
		Database::get_by_prefix(self, col, prefix)
	}

	fn write_buffered(&self, transaction: DBTransaction) {
		Database::write_buffered(self, transaction)
	}

	fn write(&self, transaction: DBTransaction) -> io::Result<()> {
		Database::write(self, transaction)
	}

	fn flush(&self) -> io::Result<()> {
		Database::flush(self)
	}

	fn iter<'a>(&'a self, col: Option<u32>) -> Box<dyn Iterator<Item=(Box<[u8]>, Box<[u8]>)> + 'a> {
		let unboxed = Database::iter(self, col);
		Box::new(unboxed.into_iter().flat_map(|inner| inner))
	}

	fn iter_from_prefix<'a>(&'a self, col: Option<u32>, prefix: &'a [u8])
		-> Box<dyn Iterator<Item=(Box<[u8]>, Box<[u8]>)> + 'a>
	{
		let unboxed = Database::iter_from_prefix(self, col, prefix);
		Box::new(unboxed.into_iter().flat_map(|inner| inner))
	}

	fn restore(&self, new_db: &str) -> io::Result<()> {
		Database::restore(self, new_db)
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
