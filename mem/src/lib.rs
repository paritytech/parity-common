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

extern crate clear_on_drop as cod;

use std::ops::{Deref, DerefMut};

#[cfg(feature = "volatile-erase")]
use std::ptr;

#[cfg(not(feature = "volatile-erase"))]
pub use cod::clear::Clear;

/// reexport clear_on_drop crate
pub mod clear_on_drop {
  pub use cod::*;
}

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
