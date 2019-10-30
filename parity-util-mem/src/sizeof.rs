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

//! Estimation for heapsize calculation. Usable to replace call to allocator method (for some
//! allocators or simply because we just need a deterministic cunsumption measurement).

use crate::malloc_size::{MallocShallowSizeOf, MallocSizeOf, MallocSizeOfOps, MallocUnconditionalShallowSizeOf};
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::sync::Arc;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use core::mem::{size_of, size_of_val};

#[cfg(feature = "std")]
use std::mem::{size_of, size_of_val};
#[cfg(feature = "std")]
use std::sync::Arc;

impl<T: ?Sized> MallocShallowSizeOf for Box<T> {
	fn shallow_size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
		size_of_val(&**self)
	}
}

impl MallocSizeOf for String {
	fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
		self.capacity() * size_of::<u8>()
	}
}

impl<T> MallocShallowSizeOf for Vec<T> {
	fn shallow_size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
		self.capacity() * size_of::<T>()
	}
}

impl<T> MallocUnconditionalShallowSizeOf for Arc<T> {
	fn unconditional_shallow_size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
		size_of::<T>()
	}
}
