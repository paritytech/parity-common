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

use crate::{other_io_err, DBAndColumns, DBKeyValue};
use rocksdb::{DBIterator, Direction, IteratorMode, ReadOptions};
use std::io;

/// Instantiate iterators yielding `io::Result<DBKeyValue>`s.
pub trait IterationHandler {
	type Iterator: Iterator<Item = io::Result<DBKeyValue>>;

	/// Create an `Iterator` over a `ColumnFamily` corresponding to the passed index. Takes
	/// `ReadOptions` to allow configuration of the new iterator (see
	/// https://github.com/facebook/rocksdb/blob/master/include/rocksdb/options.h#L1169).
	fn iter(self, col: u32, read_opts: ReadOptions) -> Self::Iterator;
	/// Create an `Iterator` over a `ColumnFamily` corresponding to the passed index. Takes
	/// `ReadOptions` to allow configuration of the new iterator (see
	/// https://github.com/facebook/rocksdb/blob/master/include/rocksdb/options.h#L1169).
	/// The `Iterator` iterates over keys which start with the provided `prefix`.
	fn iter_with_prefix(self, col: u32, prefix: &[u8], read_opts: ReadOptions) -> Self::Iterator;
}

impl<'a> IterationHandler for &'a DBAndColumns {
	type Iterator = EitherIter<KvdbAdapter<DBIterator<'a>>, std::iter::Once<io::Result<DBKeyValue>>>;

	fn iter(self, col: u32, read_opts: ReadOptions) -> Self::Iterator {
		match self.cf(col as usize) {
			Ok(cf) => EitherIter::A(KvdbAdapter(self.db.iterator_cf_opt(cf, read_opts, IteratorMode::Start))),
			Err(e) => EitherIter::B(std::iter::once(Err(e))),
		}
	}

	fn iter_with_prefix(self, col: u32, prefix: &[u8], read_opts: ReadOptions) -> Self::Iterator {
		match self.cf(col as usize) {
			Ok(cf) => EitherIter::A(KvdbAdapter(self.db.iterator_cf_opt(
				cf,
				read_opts,
				IteratorMode::From(prefix, Direction::Forward),
			))),
			Err(e) => EitherIter::B(std::iter::once(Err(e))),
		}
	}
}

/// Small enum to avoid boxing iterators.
pub enum EitherIter<A, B> {
	A(A),
	B(B),
}

impl<A, B, I> Iterator for EitherIter<A, B>
where
	A: Iterator<Item = I>,
	B: Iterator<Item = I>,
{
	type Item = I;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::A(a) => a.next(),
			Self::B(b) => b.next(),
		}
	}
}

/// A simple wrapper that adheres to the `kvdb` interface.
pub struct KvdbAdapter<T>(T);

impl<T> Iterator for KvdbAdapter<T>
where
	T: Iterator<Item = Result<(Box<[u8]>, Box<[u8]>), rocksdb::Error>>,
{
	type Item = io::Result<DBKeyValue>;

	fn next(&mut self) -> Option<Self::Item> {
		self.0
			.next()
			.map(|r| r.map_err(other_io_err).map(|(k, v)| (k.into_vec().into(), v.into())))
	}
}
