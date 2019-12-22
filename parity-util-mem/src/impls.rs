// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

//! Implementation of `MallocSize` for common types :
//! - ethereum types uint and fixed hash.
//! - smallvec arrays of sizes 32, 36

use super::{MallocSizeOf, MallocSizeOfOps};

use ethereum_types::{Bloom, H128, H160, H256, H264, H32, H512, H520, H64, U128, U256, U512, U64};
use smallvec::SmallVec;

#[cfg(not(feature = "std"))]
use core as std;

#[cfg(feature = "std")]
malloc_size_of_is_0!(std::time::Instant);
malloc_size_of_is_0!(std::time::Duration);

malloc_size_of_is_0!(U64, U128, U256, U512, H32, H64, H128, H160, H256, H264, H512, H520, Bloom);

malloc_size_of_is_0!(
	[u8; 1],
	[u8; 2],
	[u8; 3],
	[u8; 4],
	[u8; 5],
	[u8; 6],
	[u8; 7],
	[u8; 8],
	[u8; 9],
	[u8; 10],
	[u8; 11],
	[u8; 12],
	[u8; 13],
	[u8; 14],
	[u8; 15],
	[u8; 16],
	[u8; 17],
	[u8; 18],
	[u8; 19],
	[u8; 20],
	[u8; 21],
	[u8; 22],
	[u8; 23],
	[u8; 24],
	[u8; 25],
	[u8; 26],
	[u8; 27],
	[u8; 28],
	[u8; 29],
	[u8; 30],
	[u8; 31],
	[u8; 32]
);


macro_rules! impl_smallvec {
	($size: expr) => {
		impl<T> MallocSizeOf for SmallVec<[T; $size]>
		where
			T: MallocSizeOf,
		{
			fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
				let mut n = if self.spilled() { self.capacity() * core::mem::size_of::<T>() } else { 0 };
				for elem in self.iter() {
					n += elem.size_of(ops);
				}
				n
			}
		}
	};
}

impl_smallvec!(32); // kvdb uses this
impl_smallvec!(36); // trie-db uses this

#[cfg(test)]
mod tests {
	use crate::{allocators::new_malloc_size_ops, MallocSizeOf, MallocSizeOfOps};
	use smallvec::SmallVec;
	use std::mem;
	impl_smallvec!(3);

	#[test]
	fn test_smallvec_stack_allocated_type() {
		let mut v: SmallVec<[u8; 3]> = SmallVec::new();
		let mut ops = new_malloc_size_ops();
		assert_eq!(v.size_of(&mut ops), 0);
		v.push(1);
		v.push(2);
		v.push(3);
		assert_eq!(v.size_of(&mut ops), 0);
		assert!(!v.spilled());
		v.push(4);
		assert!(v.spilled(), "SmallVec spills when going beyond the capacity of the inner backing array");
		assert_eq!(v.size_of(&mut ops), 4); // 4 u8s on the heap
	}

	#[test]
	fn test_smallvec_boxed_stack_allocated_type() {
		let mut v: SmallVec<[Box<u8>; 3]> = SmallVec::new();
		let mut ops = new_malloc_size_ops();
		assert_eq!(v.size_of(&mut ops), 0);
		v.push(Box::new(1u8));
		v.push(Box::new(2u8));
		v.push(Box::new(3u8));
		assert!(v.size_of(&mut ops) >= 3);
		assert!(!v.spilled());
		v.push(Box::new(4u8));
		assert!(v.spilled(), "SmallVec spills when going beyond the capacity of the inner backing array");
		let mut ops = new_malloc_size_ops();
		let expected_min_allocs = mem::size_of::<Box<u8>>() * 4 + 4;
		assert!(v.size_of(&mut ops) >= expected_min_allocs);
	}

	#[test]
	fn test_smallvec_heap_allocated_type() {
		let mut v: SmallVec<[String; 3]> = SmallVec::new();
		let mut ops = new_malloc_size_ops();
		assert_eq!(v.size_of(&mut ops), 0);
		v.push("COW".into());
		v.push("PIG".into());
		v.push("DUCK".into());
		assert!(!v.spilled());
		assert!(v.size_of(&mut ops) >= "COW".len() + "PIG".len() + "DUCK".len());
		v.push("ÖWL".into());
		assert!(v.spilled());
		let mut ops = new_malloc_size_ops();
		let expected_min_allocs = mem::size_of::<String>() * 4 + "ÖWL".len() + "COW".len() + "PIG".len() + "DUCK".len();
		assert!(v.size_of(&mut ops) >= expected_min_allocs);
	}
}
