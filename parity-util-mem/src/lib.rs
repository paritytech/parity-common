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

//! Crate for parity memory management related utilities.
//! It includes global allocator choice, heap measurement and
//! memory erasure.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

use malloc_size_of_derive as malloc_size_derive;


cfg_if::cfg_if! {
	if #[cfg(all(
		feature = "jemalloc-global",
		not(target_os = "windows"),
		not(target_arch = "wasm32")
	))] {
		#[global_allocator]
		/// Global allocator
		pub static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
	} else if #[cfg(feature = "dlmalloc-global")] {
		#[global_allocator]
		/// Global allocator
		pub static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;
	} else if #[cfg(feature = "weealloc-global")] {
		#[global_allocator]
		/// Global allocator
		pub static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
	} else if #[cfg(all(
			feature = "mimalloc-global",
			not(target_arch = "wasm32")
		))] {
		#[global_allocator]
		/// Global allocator
		pub static ALLOC: mimallocator::Mimalloc = mimallocator::Mimalloc;
	} else {
		// default allocator used
	}
}

pub mod allocators;

#[cfg(any(
	all(
		target_os = "macos",
		not(feature = "jemalloc-global"),
	),
	feature = "estimate-heapsize"
))]
pub mod sizeof;

/// This is a copy of patched crate `malloc_size_of` as a module.
/// We need to have it as an inner module to be able to define our own traits implementation,
/// if at some point the trait become standard enough we could use the right way of doing it
/// by implementing it in our type traits crates. At this time moving this trait to the primitive
/// types level would impact too much of the dependencies to be easily manageable.
#[macro_use] mod malloc_size;

#[cfg(feature = "ethereum-impls")]
pub mod impls;

pub use malloc_size_derive::*;
pub use malloc_size::{
 	MallocSizeOfOps,
	MallocSizeOf,
};
pub use allocators::MallocSizeOfExt;

#[cfg(feature = "std")]
#[cfg(test)]
mod test {
	use std::sync::Arc;
	use super::MallocSizeOfExt;

	#[test]
	fn test_arc() {
		let val = Arc::new("test".to_string());
		let s = val.malloc_size_of();
		assert!(s > 0);
	}
}
