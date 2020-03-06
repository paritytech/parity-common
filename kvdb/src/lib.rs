// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Key-Value store abstraction.

use bytes::Bytes;
use smallvec::SmallVec;
use std::io;

mod io_stats;

/// Required length of prefixes.
pub const PREFIX_LEN: usize = 12;

/// Database value.
pub type DBValue = Vec<u8>;
/// Database keys.
pub type DBKey = SmallVec<[u8; 32]>;

pub use io_stats::{IoStats, Kind as IoStatsKind};

/// Write transaction. Batches a sequence of put/delete operations for efficiency.
#[derive(Default, Clone, PartialEq)]
pub struct DBTransaction {
	/// Database operations.
	pub ops: Vec<DBOp>,
}

/// Database operation.
#[derive(Clone, PartialEq)]
pub enum DBOp {
	Insert { col: u32, key: DBKey, value: DBValue },
	Delete { col: u32, key: DBKey },
}

impl DBOp {
	/// Returns the key associated with this operation.
	pub fn key(&self) -> &[u8] {
		match *self {
			DBOp::Insert { ref key, .. } => key,
			DBOp::Delete { ref key, .. } => key,
		}
	}

	/// Returns the column associated with this operation.
	pub fn col(&self) -> u32 {
		match *self {
			DBOp::Insert { col, .. } => col,
			DBOp::Delete { col, .. } => col,
		}
	}
}

impl DBTransaction {
	/// Create new transaction.
	pub fn new() -> DBTransaction {
		DBTransaction::with_capacity(256)
	}

	/// Create new transaction with capacity.
	pub fn with_capacity(cap: usize) -> DBTransaction {
		DBTransaction { ops: Vec::with_capacity(cap) }
	}

	/// Insert a key-value pair in the transaction. Any existing value will be overwritten upon write.
	pub fn put(&mut self, col: u32, key: &[u8], value: &[u8]) {
		self.ops.push(DBOp::Insert { col, key: DBKey::from_slice(key), value: value.to_vec() })
	}

	/// Insert a key-value pair in the transaction. Any existing value will be overwritten upon write.
	pub fn put_vec(&mut self, col: u32, key: &[u8], value: Bytes) {
		self.ops.push(DBOp::Insert { col, key: DBKey::from_slice(key), value });
	}

	/// Delete value by key.
	pub fn delete(&mut self, col: u32, key: &[u8]) {
		self.ops.push(DBOp::Delete { col, key: DBKey::from_slice(key) });
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
pub trait KeyValueDB: Sync + Send + parity_util_mem::MallocSizeOf {
	/// Helper to create a new transaction.
	fn transaction(&self) -> DBTransaction {
		DBTransaction::new()
	}

	/// Get a value by key.
	fn get(&self, col: u32, key: &[u8]) -> io::Result<Option<DBValue>>;

	/// Get a value by partial key. Only works for flushed data.
	fn get_by_prefix(&self, col: u32, prefix: &[u8]) -> Option<Box<[u8]>>;

	/// Write a transaction of changes to the buffer.
	fn write_buffered(&self, transaction: DBTransaction);

	/// Write a transaction of changes to the backing store.
	fn write(&self, transaction: DBTransaction) -> io::Result<()> {
		self.write_buffered(transaction);
		self.flush()
	}

	/// Flush all buffered data.
	fn flush(&self) -> io::Result<()>;

	/// Iterate over flushed data for a given column.
	fn iter<'a>(&'a self, col: u32) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a>;

	/// Iterate over flushed data for a given column, starting from a given prefix.
	fn iter_from_prefix<'a>(
		&'a self,
		col: u32,
		prefix: &'a [u8],
	) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a>;

	/// Attempt to replace this database with a new one located at the given path.
	fn restore(&self, new_db: &str) -> io::Result<()>;

	/// Query statistics.
	///
	/// Not all kvdb implementations are able or expected to implement this, so by
	/// default, empty statistics is returned. Also, not all kvdb implementation
	/// can return every statistic or configured to do so (some statistics gathering
	/// may impede the performance and might be off by default).
	fn io_stats(&self, _kind: IoStatsKind) -> IoStats {
		IoStats::empty()
	}
}
