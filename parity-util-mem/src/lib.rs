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
#![cfg_attr(not(feature = "std"), feature(core_intrinsics))]
#![cfg_attr(not(feature = "std"), feature(alloc))]

#[macro_use]
extern crate cfg_if;

#[cfg(not(feature = "std"))]
extern crate alloc;

extern crate clear_on_drop as cod;

#[macro_use] extern crate malloc_size_of_derive as malloc_size_derive;

use std::ops::{Deref, DerefMut};

#[cfg(feature = "volatile-erase")]
use std::ptr;

#[cfg(not(feature = "volatile-erase"))]
pub use cod::clear::Clear;


cfg_if! {
	if #[cfg(all(
		feature = "jemalloc-global",
		feature = "jemalloc-global",
		not(target_os = "windows"),
		not(target_arch = "wasm32")
	))] {
		extern crate jemallocator;
		#[global_allocator]
		/// Global allocator
		pub static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
	} else if #[cfg(feature = "dlmalloc-global")] {
		extern crate dlmalloc;
		#[global_allocator]
		/// Global allocator
		pub static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;
	} else if #[cfg(feature = "weealloc-global")] {
		extern crate wee_alloc;
		#[global_allocator]
		/// Global allocator
		pub static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
	} else {
		// default allocator used
	}
}

pub mod allocators;

#[cfg(feature = "estimate-heapsize")]
pub mod sizeof;

#[cfg(not(feature = "std"))]
use core as std;

/// This is a copy of patched crate `malloc_size_of` as a module.
/// We need to have it as an inner module to be able to define our own traits implementation,
/// if at some point the trait become standard enough we could use the right way of doing it
/// by implementing it in our type traits crates. At this time moving this trait to the primitive
/// types level would impact too much of the dependencies to be easily manageable.
#[macro_use] mod malloc_size;

#[cfg(feature = "ethereum-impls")]
pub mod impls;

/// Reexport clear_on_drop crate.
pub mod clear_on_drop {
	pub use cod::*;
}

pub use malloc_size_derive::*;
pub use malloc_size::{
 	MallocSizeOfOps,
	MallocSizeOf,
};
pub use allocators::MallocSizeOfExt;

/// Wrapper to zero out memory when dropped.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Memzero<T: AsMut<[u8]>> {
	mem: T,
}

impl<T: AsMut<[u8]>> From<T> for Memzero<T> {
	fn from(mem: T) -> Memzero<T> {
		Memzero { mem }
	}
}

#[cfg(feature = "volatile-erase")]
impl<T: AsMut<[u8]>> Drop for Memzero<T> {
	fn drop(&mut self) {
		unsafe {
			for byte_ref in self.mem.as_mut() {
				ptr::write_volatile(byte_ref, 0)
			}
		}
	}
}

#[cfg(not(feature = "volatile-erase"))]
impl<T: AsMut<[u8]>> Drop for Memzero<T> {
	fn drop(&mut self) {
		self.as_mut().clear();
	}
}

impl<T: AsMut<[u8]>> Deref for Memzero<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.mem
	}
}

impl<T: AsMut<[u8]>> DerefMut for Memzero<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.mem
	}
}

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
