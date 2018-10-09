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

//! Reference-counted memory-based `HashDB` implementation.
extern crate hashdb;
extern crate heapsize;
extern crate rlp;
#[cfg(test)] extern crate keccak_hasher;
#[cfg(test)] extern crate tiny_keccak;
#[cfg(test)] extern crate ethereum_types;

use hashdb::{HashDB, Hasher as KeyHasher, AsHashDB};
use heapsize::HeapSizeOf;
use rlp::NULL_RLP;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash;
use std::mem;

// Backing `HashMap` parametrized with a `Hasher` for the keys `Hasher::Out` and the `Hasher::StdHasher`
// as hash map builder.
type FastMap<H, T> = HashMap<<H as KeyHasher>::Out, T, hash::BuildHasherDefault<<H as KeyHasher>::StdHasher>>;

/// Reference-counted memory-based `HashDB` implementation.
///
/// Use `new()` to create a new database. Insert items with `insert()`, remove items
/// with `remove()`, check for existence with `contains()` and lookup a hash to derive
/// the data with `get()`. Clear with `clear()` and purge the portions of the data
/// that have no references with `purge()`.
///
/// # Example
/// ```rust
/// extern crate hashdb;
/// extern crate keccak_hasher;
/// extern crate memorydb;
///
/// use hashdb::*;
/// use keccak_hasher::KeccakHasher;
/// use memorydb::*;
/// fn main() {
///   let mut m = MemoryDB::<KeccakHasher, Vec<u8>>::new();
///   let d = "Hello world!".as_bytes();
///
///   let k = m.insert(d);
///   assert!(m.contains(&k));
///   assert_eq!(m.get(&k).unwrap(), d);
///
///   m.insert(d);
///   assert!(m.contains(&k));
///
///   m.remove(&k);
///   assert!(m.contains(&k));
///
///   m.remove(&k);
///   assert!(!m.contains(&k));
///
///   m.remove(&k);
///   assert!(!m.contains(&k));
///
///   m.insert(d);
///   assert!(!m.contains(&k));

///   m.insert(d);
///   assert!(m.contains(&k));
///   assert_eq!(m.get(&k).unwrap(), d);
///
///   m.remove(&k);
///   assert!(!m.contains(&k));
/// }
/// ```
#[derive(Clone, PartialEq)]
pub struct MemoryDB<H: KeyHasher, T> {
	data: FastMap<H, (T, i32)>,
	hashed_null_node: H::Out,
	null_node_data: T,
}

impl<'a, H, T> Default for MemoryDB<H, T>
where
	H: KeyHasher,
	H::Out: HeapSizeOf,
	T: From<&'a [u8]> + Clone
{
	fn default() -> Self { Self::new() }
}

impl<'a, H, T> MemoryDB<H, T>
where
	H: KeyHasher,
	H::Out: HeapSizeOf,
	T: From<&'a [u8]> + Clone,
{
	/// Create a new instance of the memory DB.
	pub fn new() -> Self {
		MemoryDB::from_null_node(&NULL_RLP, NULL_RLP.as_ref().into())
	}
}

impl<H, T> MemoryDB<H, T>
where
	H: KeyHasher,
	H::Out: HeapSizeOf,
	T: Default,
{
	/// Remove an element and delete it from storage if reference count reaches zero.
	/// If the value was purged, return the old value.
	pub fn remove_and_purge(&mut self, key: &<H as KeyHasher>::Out) -> Option<T> {
		if key == &self.hashed_null_node {
			return None;
		}
		match self.data.entry(key.clone()) {
			Entry::Occupied(mut entry) =>
				if entry.get().1 == 1 {
					Some(entry.remove().0)
				} else {
					entry.get_mut().1 -= 1;
					None
				},
			Entry::Vacant(entry) => {
				entry.insert((T::default(), -1)); // FIXME: shouldn't it be purged?
				None
			}
		}
	}
}

impl<H: KeyHasher, T: Clone> MemoryDB<H, T> {

	/// Create a new `MemoryDB` from a given null key/data
	pub fn from_null_node(null_key: &[u8], null_node_data: T) -> Self {
		MemoryDB {
			data: FastMap::<H,_>::default(),
			hashed_null_node: H::hash(null_key),
			null_node_data,
		}
	}

	/// Clear all data from the database.
	///
	/// # Examples
	/// ```rust
	/// extern crate hashdb;
	/// extern crate keccak_hasher;
	/// extern crate memorydb;
	///
	/// use hashdb::*;
	/// use keccak_hasher::KeccakHasher;
	/// use memorydb::*;
	///
	/// fn main() {
	///   let mut m = MemoryDB::<KeccakHasher, Vec<u8>>::new();
	///   let hello_bytes = "Hello world!".as_bytes();
	///   let hash = m.insert(hello_bytes);
	///   assert!(m.contains(&hash));
	///   m.clear();
	///   assert!(!m.contains(&hash));
	/// }
	/// ```
	pub fn clear(&mut self) {
		self.data.clear();
	}

	/// Purge all zero-referenced data from the database.
	pub fn purge(&mut self) {
		self.data.retain(|_, &mut (_, rc)| rc != 0);
	}

	/// Return the internal map of hashes to data, clearing the current state.
	pub fn drain(&mut self) -> FastMap<H, (T, i32)> {
		mem::replace(&mut self.data, FastMap::<H,_>::default())
	}

	/// Grab the raw information associated with a key. Returns None if the key
	/// doesn't exist.
	///
	/// Even when Some is returned, the data is only guaranteed to be useful
	/// when the refs > 0.
	pub fn raw(&self, key: &<H as KeyHasher>::Out) -> Option<(T, i32)> {
		if key == &self.hashed_null_node {
			return Some((self.null_node_data.clone(), 1));
		}
		self.data.get(key).map(|(value, count)| (value.clone(), *count))
	}

	/// Consolidate all the entries of `other` into `self`.
	pub fn consolidate(&mut self, mut other: Self) {
		for (key, (value, rc)) in other.drain() {
			match self.data.entry(key) {
				Entry::Occupied(mut entry) => {
					if entry.get().1 < 0 {
						entry.get_mut().0 = value;
					}

					entry.get_mut().1 += rc;
				}
				Entry::Vacant(entry) => {
					entry.insert((value, rc));
				}
			}
		}
	}
}

impl<H, T> MemoryDB<H, T>
where
	H: KeyHasher,
	H::Out: HeapSizeOf,
	T: HeapSizeOf,
{
	/// Returns the size of allocated heap memory
	pub fn mem_used(&self) -> usize {
		self.data.heap_size_of_children()
	}
}

impl<H, T> HashDB<H, T> for MemoryDB<H, T>
where
	H: KeyHasher,
	T: Default + PartialEq<T> + for<'a> From<&'a [u8]> + Send + Sync + Clone,
{
	fn keys(&self) -> HashMap<H::Out, i32> {
		self.data.iter()
			.filter_map(|(k, v)| if v.1 != 0 {
				Some((*k, v.1))
			} else {
				None
			})
			.collect()
	}

	fn get(&self, key: &H::Out) -> Option<T> {
		if key == &self.hashed_null_node {
			return Some(self.null_node_data.clone());
		}

		match self.data.get(key) {
			Some(&(ref d, rc)) if rc > 0 => Some(d.clone()),
			_ => None
		}
	}

	fn contains(&self, key: &H::Out) -> bool {
		if key == &self.hashed_null_node {
			return true;
		}

		match self.data.get(key) {
			Some(&(_, x)) if x > 0 => true,
			_ => false
		}
	}

	fn emplace(&mut self, key:H::Out, value: T) {
		if value == self.null_node_data {
			return;
		}

		match self.data.entry(key) {
			Entry::Occupied(mut entry) => {
				let &mut (ref mut old_value, ref mut rc) = entry.get_mut();
				if *rc <= 0 {
					*old_value = value;
				}
				*rc += 1;
			},
			Entry::Vacant(entry) => {
				entry.insert((value, 1));
			},
		}
	}

	fn insert(&mut self, value: &[u8]) -> H::Out {
		if value == &NULL_RLP {
			return self.hashed_null_node.clone();
		}
		let key = H::hash(value);
		match self.data.entry(key) {
			Entry::Occupied(mut entry) => {
				let &mut (ref mut old_value, ref mut rc) = entry.get_mut();
				if *rc <= 0 {
					*old_value = value.into();
				}
				*rc += 1;
			},
			Entry::Vacant(entry) => {
				entry.insert((value.into(), 1));
			},
		}
		key
	}

	fn remove(&mut self, key: &H::Out) {
		if key == &self.hashed_null_node {
			return;
		}

		match self.data.entry(*key) {
			Entry::Occupied(mut entry) => {
				let &mut (_, ref mut rc) = entry.get_mut();
				*rc -= 1;
			},
			Entry::Vacant(entry) => {
				entry.insert((T::default(), -1));
			},
		}
	}

}

impl<H, T> AsHashDB<H, T> for MemoryDB<H, T>
where
	H: KeyHasher,
	T: Default + PartialEq<T> + for<'a> From<&'a[u8]> + Send + Sync + Clone,
{
	fn as_hashdb(&self) -> &HashDB<H, T> { self }
	fn as_hashdb_mut(&mut self) -> &mut HashDB<H, T> { self }
}

#[cfg(test)]
mod tests {
	use super::*;
	use tiny_keccak::Keccak;
	use ethereum_types::H256;
	use keccak_hasher::KeccakHasher;

	#[test]
	fn memorydb_remove_and_purge() {
		let hello_bytes = b"Hello world!";
		let mut hello_key = [0;32];
		Keccak::keccak256(hello_bytes, &mut hello_key);
		let hello_key = H256(hello_key);

		let mut m = MemoryDB::<KeccakHasher, Vec<u8>>::new();
		m.remove(&hello_key);
		assert_eq!(m.raw(&hello_key).unwrap().1, -1);
		m.purge();
		assert_eq!(m.raw(&hello_key).unwrap().1, -1);
		m.insert(hello_bytes);
		assert_eq!(m.raw(&hello_key).unwrap().1, 0);
		m.purge();
		assert_eq!(m.raw(&hello_key), None);

		let mut m = MemoryDB::<KeccakHasher, Vec<u8>>::new();
		assert!(m.remove_and_purge(&hello_key).is_none());
		assert_eq!(m.raw(&hello_key).unwrap().1, -1);
		m.insert(hello_bytes);
		m.insert(hello_bytes);
		assert_eq!(m.raw(&hello_key).unwrap().1, 1);
		assert_eq!(&*m.remove_and_purge(&hello_key).unwrap(), hello_bytes);
		assert_eq!(m.raw(&hello_key), None);
		assert!(m.remove_and_purge(&hello_key).is_none());
	}

	#[test]
	fn consolidate() {
		let mut main = MemoryDB::<KeccakHasher, Vec<u8>>::new();
		let mut other = MemoryDB::<KeccakHasher, Vec<u8>>::new();
		let remove_key = other.insert(b"doggo");
		main.remove(&remove_key);

		let insert_key = other.insert(b"arf");
		main.emplace(insert_key, "arf".as_bytes().to_vec());

		let negative_remove_key = other.insert(b"negative");
		other.remove(&negative_remove_key);	// ref cnt: 0
		other.remove(&negative_remove_key);	// ref cnt: -1
		main.remove(&negative_remove_key);	// ref cnt: -1

		main.consolidate(other);

		let overlay = main.drain();

		assert_eq!(overlay.get(&remove_key).unwrap(), &("doggo".as_bytes().to_vec(), 0));
		assert_eq!(overlay.get(&insert_key).unwrap(), &("arf".as_bytes().to_vec(), 2));
		assert_eq!(overlay.get(&negative_remove_key).unwrap(), &("negative".as_bytes().to_vec(), -2));
	}

	#[test]
	fn default_works() {
		let mut db = MemoryDB::<KeccakHasher, Vec<u8>>::default();
		let hashed_null_node = KeccakHasher::hash(&NULL_RLP);
		assert_eq!(db.insert(&NULL_RLP), hashed_null_node);
	}
}
