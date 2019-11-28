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

//! This module contains an implementation of a RocksDB iterator
//! wrapped inside a `RwLock`. Since `RwLock` "owns" the inner data,
//! we're using `owning_ref` to work around the borrowing rules of Rust.

use crate::DBAndColumns;
use owning_ref::{OwningHandle, StableAddress};
use parking_lot::RwLockReadGuard;
use rocksdb::{DBIterator, IteratorMode};
use std::ops::{Deref, DerefMut};

/// A tuple holding key and value data, used as the iterator item type.
pub type KeyValuePair = (Box<[u8]>, Box<[u8]>);

/// Iterator with built-in synchronization.
pub struct ReadGuardedIterator<'a, I, T> {
	inner: OwningHandle<UnsafeStableAddress<'a, Option<T>>, DerefWrapper<Option<I>>>,
}

// We can't implement `StableAddress` for a `RwLockReadGuard`
// directly due to orphan rules.
#[repr(transparent)]
struct UnsafeStableAddress<'a, T>(RwLockReadGuard<'a, T>);

impl<'a, T> Deref for UnsafeStableAddress<'a, T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		self.0.deref()
	}
}

// RwLockReadGuard dereferences to a stable address; qed
unsafe impl<'a, T> StableAddress for UnsafeStableAddress<'a, T> {}

struct DerefWrapper<T>(T);

impl<T> Deref for DerefWrapper<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> DerefMut for DerefWrapper<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<'a, I: Iterator, T> Iterator for ReadGuardedIterator<'a, I, T> {
	type Item = I::Item;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.deref_mut().as_mut().and_then(|iter| iter.next())
	}
}

/// Instantiate iterators yielding `KeyValuePair`s.
pub trait IterationHandler {
	type Iterator: Iterator<Item = KeyValuePair>;

	/// Create an `Iterator` over the default DB column or over a `ColumnFamily` if a column number
	/// is passed.
	fn iter(&self, col: Option<u32>) -> Self::Iterator;
	/// Create an `Iterator` over the default DB column or over a `ColumnFamily` if a column number
	/// is passed. The iterator starts from the first key having the provided `prefix`.
	fn iter_from_prefix(&self, col: Option<u32>, prefix: &[u8]) -> Self::Iterator;
}

impl<'a, T> ReadGuardedIterator<'a, <&'a T as IterationHandler>::Iterator, T>
where
	&'a T: IterationHandler,
{
	pub fn new(read_lock: RwLockReadGuard<'a, Option<T>>, col: Option<u32>) -> Self {
		Self { inner: Self::new_inner(read_lock, |db| db.iter(col)) }
	}

	pub fn new_from_prefix(read_lock: RwLockReadGuard<'a, Option<T>>, col: Option<u32>, prefix: &[u8]) -> Self {
		Self { inner: Self::new_inner(read_lock, |db| db.iter_from_prefix(col, prefix)) }
	}

	fn new_inner(
		rlock: RwLockReadGuard<'a, Option<T>>,
		f: impl FnOnce(&'a T) -> <&'a T as IterationHandler>::Iterator,
	) -> OwningHandle<UnsafeStableAddress<'a, Option<T>>, DerefWrapper<Option<<&'a T as IterationHandler>::Iterator>>> {
		OwningHandle::new_with_fn(UnsafeStableAddress(rlock), move |rlock| {
			let rlock = unsafe { rlock.as_ref().expect("initialized as non-null; qed") };
			DerefWrapper(rlock.as_ref().map(f))
		})
	}
}

impl<'a> IterationHandler for &'a DBAndColumns {
	type Iterator = DBIterator<'a>;

	fn iter(&self, col: Option<u32>) -> Self::Iterator {
		col.map_or_else(
			|| self.db.iterator(IteratorMode::Start),
			|c| {
				self.db
					.iterator_cf(self.get_colf(c as usize), IteratorMode::Start)
					.expect("iterator params are valid; qed")
			},
		)
	}

	fn iter_from_prefix(&self, col: Option<u32>, prefix: &[u8]) -> Self::Iterator {
		col.map_or_else(
			|| self.db.prefix_iterator(prefix),
			|c| self.db.prefix_iterator_cf(self.get_colf(c as usize), prefix).expect("iterator params are valid; qed"),
		)
	}
}
