// Copyright 2020 Parity Technologies (UK) Ltd.
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

//! Async Key-Value store abstraction.

pub use kvdb::*;
use futures::prelude::{Future, Stream};
use std::pin::Pin;
use std::io;

/// Generic async key-value database.
///
/// The API laid out here, along with the `Sync` bound implies interior synchronization for
/// implementation.
pub trait AsyncKeyValueDB: Sync + Send + parity_util_mem::MallocSizeOf {
	/// Helper to create a new transaction.
	fn transaction(&self) -> DBTransaction {
		DBTransaction::new()
	}

	/// Get a value by key.
	fn get(&self, col: u32, key: &[u8]) -> Pin<Box<dyn Future<Output = io::Result<Option<DBValue>>> + Send>>;

	/// Get a value by partial key.
	fn get_by_prefix(&self, col: u32, prefix: &[u8]) -> Pin<Box<dyn Future<Output = Option<Box<[u8]>>> + Send>>;

	/// Write a transaction of changes to the backing store.
	fn write(&self, transaction: DBTransaction) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send>>;

	/// Iterate over the data for a given column.
	fn iter<'a>(&'a self, col: u32) -> Pin<Box<dyn Stream<Item = (Box<[u8]>, Box<[u8]>)> + 'a + Send>>;

	/// Iterate over the data for a given column, starting from a given prefix.
	fn iter_from_prefix<'a>(
		&'a self,
		col: u32,
		prefix: &'a [u8],
	) -> Pin<Box<dyn Stream<Item = (Box<[u8]>, Box<[u8]>)> + 'a + Send>>;
}

impl<T: ?Sized + KeyValueDB> AsyncKeyValueDB for T {
	fn get(&self, col: u32, key: &[u8]) -> Pin<Box<dyn Future<Output = io::Result<Option<DBValue>>> + Send>> {
		Box::pin(futures::future::ready(self.get(col, key)))
	}
	fn get_by_prefix(&self, col: u32, prefix: &[u8]) -> Pin<Box<dyn Future<Output = Option<Box<[u8]>>> + Send>> {
		Box::pin(futures::future::ready(self.get_by_prefix(col, prefix)))
	}
	fn write(&self, transaction: DBTransaction) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send>> {
		Box::pin(futures::future::ready(self.write(transaction)))
	}
	fn iter<'a>(&'a self, col: u32) -> Pin<Box<dyn Stream<Item = (Box<[u8]>, Box<[u8]>)> + 'a + Send>> {
		Box::pin(futures::stream::iter(self.iter(col)))
	}
	fn iter_from_prefix<'a>(
		&'a self,
		col: u32,
		prefix: &'a [u8],
	) -> Pin<Box<dyn Stream<Item = (Box<[u8]>, Box<[u8]>)> + 'a + Send>> {
		Box::pin(futures::stream::iter(self.iter_from_prefix(col, prefix)))
	}
}
