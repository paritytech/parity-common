// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Key-Value store abstraction with `RocksDB` backend.

use std::io;
use std::path::Path;
use std::sync::Arc;

/// Required length of prefixes.
pub const PREFIX_LEN: usize = 12;

/// Write transaction. Batches a sequence of put/delete operations for efficiency.
#[derive(Default, Clone, PartialEq)]
pub struct DBTransaction<K, V> {
	/// Database operations.
	pub ops: Vec<DBOp<K, V>>,
}

/// Database operation.
#[derive(Clone, PartialEq)]
pub enum DBOp<K, V> {
	Insert {
		col: Option<u32>,
		key: K,
		value: V,
	},
	Delete {
		col: Option<u32>,
		key: K,
	}
}

impl<K, V> DBOp<K, V> {
	/// Returns the key associated with this operation.
	pub fn key(&self) -> &[u8] where K: AsRef<[u8]> {
		match *self {
			DBOp::Insert { ref key, .. } => key.as_ref(),
			DBOp::Delete { ref key, .. } => key.as_ref(),
		}
	}

	/// Returns the column associated with this operation.
	pub fn col(&self) -> Option<u32> {
		match *self {
			DBOp::Insert { col, .. } => col,
			DBOp::Delete { col, .. } => col,
		}
	}
}

impl<K, V> DBTransaction<K, V> {
	/// Create new transaction.
	pub fn new() -> Self {
		DBTransaction::with_capacity(256)
	}

	/// Create new transaction with capacity.
	pub fn with_capacity(cap: usize) -> Self {
		DBTransaction {
			ops: Vec::with_capacity(cap)
		}
	}

	/// Insert a key-value pair in the transaction. Any existing value will be overwritten upon write.
	pub fn put<IK, IV>(&mut self, col: Option<u32>, key: IK, value: IV)
	where
		IK: Into<K>,
		IV: Into<V>,
	{
		self.ops.push(DBOp::Insert {
			col: col,
			key: key.into(),
			value: value.into(),
		});
	}

	/// Delete value by key.
	pub fn delete<IK: Into<K>>(&mut self, col: Option<u32>, key: IK) {
		self.ops.push(DBOp::Delete {
			col: col,
			key: key.into(),
		});
	}
}

/// Generic key-value database.
///
/// This makes a distinction between "buffered" and "flushed" values. Values which have been
/// written can always be read, but may be present in an in-memory buffer. Values which have
/// been flushed have been moved to backing storage, like a RocksDB instance. There are certain
/// operations which are only guaranteed to operate on flushed data and not buffered,
/// although implementations may differ in this regard.
///
/// The contents of an interior buffer may be explicitly flushed using the `flush` method.
///
/// The `KeyValueDB` also deals in "column families", which can be thought of as distinct
/// stores within a database. Keys written in one column family will not be accessible from
/// any other. The number of column families must be specified at initialization, with a
/// differing interface for each database. The `None` argument in place of a column index
/// is always supported.
///
/// The API laid out here, along with the `Sync` bound implies interior synchronization for
/// implementation.
pub trait KeyValueDB<K: AsRef<[u8]>, V>: Sync + Send {

	/// Helper to create a new transaction.
	fn transaction(&self) -> DBTransaction<K, V> { DBTransaction::new() }

	/// Get a value by key.
	fn get(&self, col: Option<u32>, key: &[u8]) -> io::Result<Option<V>>;

	/// Get a value by partial key. Only works for flushed data.
	fn get_by_prefix(&self, col: Option<u32>, prefix: &[u8]) -> Option<V>;

	/// Write a transaction of changes to the buffer.
	fn write_buffered(&self, transaction: DBTransaction<K, V>);

	/// Write a transaction of changes to the backing store.
	fn write(&self, transaction: DBTransaction<K, V>) -> io::Result<()> {
		self.write_buffered(transaction);
		self.flush()
	}

	/// Flush all buffered data.
	fn flush(&self) -> io::Result<()>;

	/// Iterate over flushed data for a given column.
	fn iter<'a>(&'a self, col: Option<u32>) -> Box<Iterator<Item=(K, V)> + 'a>;

	/// Iterate over flushed data for a given column, starting from a given prefix.
	fn iter_from_prefix<'a>(&'a self, col: Option<u32>, prefix: &'a [u8])
		-> Box<Iterator<Item=(K, V)> + 'a>;

	/// Attempt to replace this database with a new one located at the given path.
	fn restore<P: AsRef<Path>>(&self, new_db: P) -> io::Result<()>;
}

/// Generic key-value database handler. This trait contains one function `open`. When called, it opens database with a
/// predefined config.
pub trait KeyValueDBHandler<K: AsRef<[u8]>, V>: Send + Sync {
	type DB: KeyValueDB<K, V>;
	/// Open the predefined key-value database.
	fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<Arc<Self::DB>>;
}
