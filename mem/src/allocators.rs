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
//!	 - no features: default implementation from servo `heapsize` crate
//!	 - weealloc: return 0 for compatibility
//!	 - dlmalloc: enable, does not work, for compatibility only
//!	 - jemalloc: compile error
//! - arch x86:
//!	 - no features: use default alloc
//!	 - jemalloc: use jemallocator crate
//!	 - weealloc: return 0 for compatibility
//!	 - dlmalloc: enable, does not work, for compatibility only
//! - arch wasm32:
//!	 - no features: return 0 for compatibility
//!	 - dlmalloc: return 0 for compatibility (usable_size could be implemented if needed in
//!	 dlmalloc crate)
//!	 - weealloc: return 0 for compatibility
//!	 - jemalloc: compile error


use malloc_size::{MallocSizeOfOps, VoidPtrToSizeFn, MallocSizeOf};
#[cfg(feature = "conditional-mettering")]
use malloc_size::{MallocConditionalSizeOf, VoidPtrToBoolFnMut};
#[cfg(feature = "std")]
#[cfg(not(feature = "conditional-mettering"))]
use malloc_size::MallocUnconditionalSizeOf;
#[cfg(feature = "std")]
use std::os::raw::c_void;
#[cfg(not(feature = "std"))]
use core::ffi::c_void;
#[cfg(not(feature = "std"))]
use alloc::collections::btree_set::BTreeSet;
#[cfg(not(feature = "std"))]
#[cfg(feature = "conditional-mettering")]
pub use alloc::boxed::Box;
#[cfg(not(feature = "weealloc-global"))]
#[cfg(not(feature = "dlmalloc-global"))]
#[cfg(not(feature = "jemalloc-global"))]
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

#[cfg(not(feature = "weealloc-global"))]
#[cfg(not(feature = "dlmalloc-global"))]
#[cfg(not(feature = "jemalloc-global"))]
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

#[cfg(not(feature = "weealloc-global"))]
#[cfg(not(feature = "dlmalloc-global"))]
#[cfg(not(feature = "jemalloc-global"))]
#[cfg(target_arch = "wasm32")]
mod usable_size {
	use super::*;

	/// Warning this is for compatibility only TODO dlmalloc impl
	pub unsafe extern "C" fn malloc_usable_size(mut ptr: *const c_void) -> usize {
		panic!("Please run with `estimate-heapsize` feature")
	}

	#[inline]
	pub fn new_enclosing_size_fn() -> Option<VoidPtrToSizeFn> {
		None
	}
}

#[cfg(feature = "jemalloc-global")]
mod usable_size {
	use super::*;

	pub unsafe extern "C" fn malloc_usable_size(ptr: *const c_void) -> usize {
		jemallocator::usable_size(ptr)
	}

	#[inline]
	pub fn new_enclosing_size_fn() -> Option<VoidPtrToSizeFn> {
		None
	}
}

#[cfg(feature = "dlmalloc-global")]
mod usable_size {
	use super::*;

	/// Warning this is for compatibility only TODO dlmalloc impl
	pub unsafe extern "C" fn malloc_usable_size(ptr: *const c_void) -> usize {
		panic!("Running estimation this code should never be reached")
	}

	#[inline]
	pub fn new_enclosing_size_fn() -> Option<VoidPtrToSizeFn> {
		None
	}
}

#[cfg(feature = "weealloc-global")]
mod usable_size {
	use super::*;

	/// Warning this is for compatibility only
	pub unsafe extern "C" fn malloc_usable_size(ptr: *const c_void) -> usize {
		panic!("Running estimation this code should never be reached")
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

#[cfg(feature = "conditional-mettering")]
/// Get a new instance of a MallocSizeOfOps with a haveseen ptr function
pub fn new_count_malloc_size_ops(count_fn: Box<VoidPtrToBoolFnMut>) -> MallocSizeOfOps {
	MallocSizeOfOps::new(
		usable_size::malloc_usable_size,
		usable_size::new_enclosing_size_fn(),
		Some(count_fn),
	)
}

#[cfg(feature = "conditional-mettering")]
#[cfg(feature = "std")]
fn new_set<V: std::hash::Hash + Eq>() -> std::collections::HashSet<V> {
	std::collections::HashSet::new()
}

#[cfg(feature = "conditional-mettering")]
#[cfg(not(feature = "std"))]
fn new_set<V: Ord>() -> BTreeSet<V> {
	BTreeSet::new()
}


#[cfg(feature = "conditional-mettering")]
/// count function for testing purpose only (slow)
pub fn test_count() -> Box<FnMut(*const c_void) -> bool> {
	let mut set = new_set();
	Box::new(move |ptr| {
		let r = if set.contains(&ptr) {
			true
		} else {
			set.insert(ptr);
			false
		};
		r
	})
}

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
#[cfg(feature = "std")]
#[cfg(not(feature = "conditional-mettering"))]
impl<T: MallocSizeOf> MallocSizeOf for std::sync::Arc<T> {
	fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
		self.unconditional_size_of(ops)
	}
}

#[cfg(feature = "std")]
#[cfg(feature = "conditional-mettering")]
impl<T: MallocSizeOf> MallocSizeOf for std::sync::Arc<T> {
	fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
		self.conditional_size_of(ops)
	}
}
