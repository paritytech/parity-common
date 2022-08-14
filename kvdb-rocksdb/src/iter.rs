// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains an implementation of a RocksDB iterator
//! wrapped inside a `RwLock`. Since `RwLock` "owns" the inner data,
//! we're using `owning_ref` to work around the borrowing rules of Rust.
//!
//! Note: this crate does not use "Prefix Seek" mode which means that the prefix iterator
//! will return keys not starting with the given prefix as well (as long as `key >= prefix`).
//! To work around this we set an upper bound to the prefix successor.
//! See https://github.com/facebook/rocksdb/wiki/Prefix-Seek-API-Changes for details.

use crate::DBAndColumns;
use rocksdb::{DBIterator, Direction, IteratorMode, ReadOptions};

/// A tuple holding key and value data, used as the iterator item type.
pub type KeyValuePair = (Box<[u8]>, Box<[u8]>);

/// Instantiate iterators yielding `KeyValuePair`s.
pub trait IterationHandler {
	type Iterator: Iterator<Item = KeyValuePair>;

	/// Create an `Iterator` over a `ColumnFamily` corresponding to the passed index. Takes
	/// `ReadOptions` to allow configuration of the new iterator (see
	/// https://github.com/facebook/rocksdb/blob/master/include/rocksdb/options.h#L1169).
	fn iter(&self, col: u32, read_opts: ReadOptions) -> Self::Iterator;
	/// Create an `Iterator` over a `ColumnFamily` corresponding to the passed index. Takes
	/// `ReadOptions` to allow configuration of the new iterator (see
	/// https://github.com/facebook/rocksdb/blob/master/include/rocksdb/options.h#L1169).
	/// The `Iterator` iterates over keys which start with the provided `prefix`.
	fn iter_with_prefix(&self, col: u32, prefix: &[u8], read_opts: ReadOptions) -> Self::Iterator;
}

impl<'a> IterationHandler for &'a DBAndColumns {
	type Iterator = DBIterator<'a>;

	fn iter(&self, col: u32, read_opts: ReadOptions) -> Self::Iterator {
		self.db.iterator_cf_opt(self.cf(col as usize), read_opts, IteratorMode::Start)
	}

	fn iter_with_prefix(&self, col: u32, prefix: &[u8], read_opts: ReadOptions) -> Self::Iterator {
		self.db
			.iterator_cf_opt(self.cf(col as usize), read_opts, IteratorMode::From(prefix, Direction::Forward))
	}
}
