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

//! Memory related utilities.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(core_intrinsics))]
#![cfg_attr(not(feature = "std"), feature(alloc))]

#[cfg(not(feature = "std"))]
extern crate alloc;

extern crate clear_on_drop as cod;

//extern crate malloc_size_of as malloc_size;
#[macro_use] extern crate malloc_size_of_derive as malloc_size_derive;

use std::ops::{Deref, DerefMut};

#[cfg(feature = "volatile-erase")]
use std::ptr;

#[cfg(not(feature = "volatile-erase"))]
pub use cod::clear::Clear;

#[cfg(feature = "jemalloc-global")]
extern crate jemallocator;

#[cfg(feature = "dlmalloc-global")]
extern crate dlmalloc;

#[cfg(feature = "weealloc-global")]
extern crate wee_alloc;

#[cfg(feature = "jemalloc-global")]
#[global_allocator]
/// global allocator
pub static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(feature = "dlmalloc-global")]
#[global_allocator]
/// global allocator
pub static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

#[cfg(feature = "weealloc-global")]
#[global_allocator]
/// global allocator
pub static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub mod allocators;

#[cfg(feature = "estimate-heapsize")]
pub mod sizeof;

#[cfg(not(feature = "std"))]
use core as std;

/// This is a copy of patched crate `malloc_size_of` as a module.
/// We need to have it as an inner module to be able to define our own traits implementation,
/// if at some point the trait become standard enough we could use the right way of doing it
/// by implementing it in our type traits crates. At this time a move on this trait if implemented 
/// at primitive types level would impact to much of the dependency to be easilly manageable.
#[macro_use] mod malloc_size;

#[cfg(feature = "ethereum-impls")]
pub mod impls;

/// reexport clear_on_drop crate
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
