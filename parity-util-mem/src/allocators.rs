// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! default allocator management
//! Features are:
//! - windows:
//! 	 - no features: default implementation from servo `heapsize` crate
//! 	 - weealloc: default to `estimate_size`
//! 	 - dlmalloc: default to `estimate_size`
//! 	 - jemalloc: default windows allocator is used instead
//! 	 - mimalloc: use mimallocator crate
//! - arch x86:
//! 	 - no features: use default alloc
//! 	 - jemalloc: use tikv-jemallocator crate
//! 	 - weealloc: default to `estimate_size`
//! 	 - dlmalloc: default to `estimate_size`
//! 	 - mimalloc: use mimallocator crate
//! - arch x86/macos:
//! 	 - no features: use default alloc, requires using `estimate_size`
//! 	 - jemalloc: use tikv-jemallocator crate
//! 	 - weealloc: default to `estimate_size`
//! 	 - dlmalloc: default to `estimate_size`
//! 	 - mimalloc: use mimallocator crate
//! - arch wasm32:
//! 	 - no features: default to `estimate_size`
//! 	 - weealloc: default to `estimate_size`
//! 	 - dlmalloc: default to `estimate_size`
//! 	 - jemalloc: compile error
//! 	 - mimalloc: compile error (until https://github.com/microsoft/mimalloc/pull/32 is merged)

#[cfg(feature = "std")]
use crate::malloc_size::MallocUnconditionalSizeOf;
use crate::malloc_size::{MallocSizeOf, MallocSizeOfOps, VoidPtrToSizeFn};
#[cfg(not(feature = "std"))]
use core::ffi::c_void;
#[cfg(feature = "std")]
use std::os::raw::c_void;

mod usable_size {

	use super::*;

	cfg_if::cfg_if! {

		if #[cfg(any(
			target_arch = "wasm32",
			feature = "estimate-heapsize",
			feature = "weealloc-global",
			feature = "dlmalloc-global",
		))] {

			// do not try system allocator

			/// Warning this is for compatibility only.
			/// This function does panic: `estimate-heapsize` feature needs to be activated
			/// to avoid this function call.
			pub unsafe extern "C" fn malloc_usable_size(_ptr: *const c_void) -> usize {
				unreachable!("estimate heapsize only")
			}

		} else if #[cfg(target_os = "windows")] {

			use winapi::um::heapapi::{GetProcessHeap, HeapSize, HeapValidate};
			use winapi::ctypes::c_void as winapi_c_void;

			/// Get the size of a heap block.
			/// Call windows allocator through `winapi` crate
			pub unsafe extern "C" fn malloc_usable_size(mut ptr: *const c_void) -> usize {

				let heap = GetProcessHeap();

				if HeapValidate(heap, 0, ptr as *const winapi_c_void) == 0 {
					ptr = *(ptr as *const *const c_void).offset(-1);
				}

				HeapSize(heap, 0, ptr as *const winapi_c_void) as usize
			}

		} else if #[cfg(feature = "jemalloc-global")] {

			/// Use of jemalloc usable size C function through jemallocator crate call.
			pub unsafe extern "C" fn malloc_usable_size(ptr: *const c_void) -> usize {
				tikv_jemallocator::usable_size(ptr)
			}

		} else if #[cfg(feature = "mimalloc-global")] {

			/// Use of mimalloc usable size C function through mimalloc_sys crate call.
			pub unsafe extern "C" fn malloc_usable_size(ptr: *const c_void) -> usize {
				// mimalloc doesn't actually mutate the value ptr points to,
				// but requires a mut pointer in the API
				libmimalloc_sys::mi_usable_size(ptr as *mut _)
			}

		} else if #[cfg(any(
			target_os = "linux",
			target_os = "android",
			target_os = "freebsd",
		))] {
			/// Linux/BSD call system allocator (currently malloc).
			extern "C" {
				pub fn malloc_usable_size(ptr: *const c_void) -> usize;
			}

		} else {
			// default allocator for non linux or windows system use estimate
			pub unsafe extern "C" fn malloc_usable_size(_ptr: *const c_void) -> usize {
				unreachable!("estimate heapsize or feature allocator needed")
			}

		}

	}

	/// No enclosing function defined.
	#[inline]
	pub fn new_enclosing_size_fn() -> Option<VoidPtrToSizeFn> {
		None
	}
}

/// Get a new instance of a MallocSizeOfOps
pub fn new_malloc_size_ops() -> MallocSizeOfOps {
	MallocSizeOfOps::new(usable_size::malloc_usable_size, usable_size::new_enclosing_size_fn(), None)
}

/// Extension methods for `MallocSizeOf` trait, do not implement
/// directly.
/// It allows getting heapsize without exposing `MallocSizeOfOps`
/// (a single default `MallocSizeOfOps` is used for each call).
pub trait MallocSizeOfExt: MallocSizeOf {
	/// Method to launch a heapsize measurement with a
	/// fresh state.
	fn malloc_size_of(&self) -> usize {
		let mut ops = new_malloc_size_ops();
		<Self as MallocSizeOf>::size_of(self, &mut ops)
	}
}

impl<T: MallocSizeOf> MallocSizeOfExt for T {}

#[cfg(feature = "std")]
impl<T: MallocSizeOf> MallocSizeOf for std::sync::Arc<T> {
	fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
		self.unconditional_size_of(ops)
	}
}
