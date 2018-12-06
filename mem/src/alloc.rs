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

//! default allocator management
//! Features are:
//! - windows:
//!   - no features: default implementation from servo `heapsize` crate
//!   - weealloc: enable, untested
//!   - dlmalloc: enable, does not work, for compatibility only
//! - arch x86:
//!   - no features: use default alloc
//!   - jemalloc: use jemallocator crate
//!   - weealloc: enable, untested
//!   - dlmalloc: enable, does not work, for compatibility only
//! - arch wasm32:
//!   - no features: return 0 for compatibility
//!   - dlmalloc: return 0 for compatibility (usable_size could be implemented if needed in
//!   dlmalloc crate)
//!   - weealloc: enable
//!   - jemalloc: compile_error

use malloc_size::{MallocSizeOfOps, VoidPtrToSizeFn, MallocSizeOf, MallocUnconditionalSizeOf};
use std::os::raw::c_void;

#[cfg(windows)]
mod usable_size {
	#[cfg(target_os = "windows")]
	extern crate winapi;

	#[cfg(target_os = "windows")]
	use winapi::um::heapapi::{GetProcessHeap, HeapSize, HeapValidate};
	use std::os::raw::c_void;

	/// Get the size of a heap block.
	pub unsafe extern "C" fn malloc_usable_size(mut ptr: *const c_void) -> usize {

		let heap = GetProcessHeap();

		if HeapValidate(heap, 0, ptr) == 0 {
			ptr = *(ptr as *const *const c_void).offset(-1);
		}

		HeapSize(heap, 0, ptr) as usize
	}

	#[inline]
	pub fn new_enclosing_size_fn() -> Option<VoidPtrToSizeFn> {
		None
	}
}

#[cfg(all(not(windows), not(target_arch = "wasm32")))]
// default
mod usable_size {
	use super::*;

	extern "C" {
		#[cfg_attr(any(prefixed_jemalloc, target_os = "macos", target_os = "ios", target_os = "android"), link_name = "je_malloc_usable_size")]
		pub fn malloc_usable_size(ptr: *const c_void) -> usize;
	}

	#[inline]
	pub fn new_enclosing_size_fn() -> Option<VoidPtrToSizeFn> {
		None
	}

}

/// Get a new instance of a MallocSizeOfOps
pub fn new_malloc_size_ops() -> MallocSizeOfOps {
	MallocSizeOfOps::new(
		usable_size::malloc_usable_size,
		usable_size::new_enclosing_size_fn(),
		None,
	)
}

/*/// Glue trait for quick switch in parity-ethereum
/// `malloc_size_of` traits should be used next
pub trait HeapSizeOf {
	fn heap_size_of_children(&self) -> usize;
}

impl<T: MallocSizeOf> HeapSizeOf for T {
	fn heap_size_of_children(&self) -> usize {
		let mut ops = new_malloc_size_ops();

		<T as MallocSizeOf>::size_of(self, &mut ops)
	}
}*/

/// Extension methods for `MallocSizeOf`
pub trait MallocSizeOfExt: MallocSizeOf {
	fn m_size_of(&self) -> usize {
		let mut ops = new_malloc_size_ops();
		<Self as MallocSizeOf>::size_of(self, &mut ops)
	}
}

impl<T: MallocSizeOf> MallocSizeOfExt for T {}


/// we currently do not have use case where a conditional fn is use so
/// we default to unconditional mettering
/// It would be interesting to run some test with global mutex other weak handle in ops to check
/// how much we measure multiple times
impl<T: MallocSizeOf> MallocSizeOf for std::sync::Arc<T> {
  fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
    self.unconditional_size_of(ops)
  }
}
