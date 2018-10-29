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


//! memmap non breaking compile implementation, note that it is non functional

#[cfg(not(target_arch = "wasm32"))]
extern crate memmap;

#[cfg(not(target_arch = "wasm32"))]
pub use memmap::MmapMut;

#[cfg(target_arch = "wasm32")]
pub struct MmapMut;

#[cfg(target_arch = "wasm32")]
use std::io::{ErrorKind, Result};

#[cfg(target_arch = "wasm32")]
use std::fs::File;

#[cfg(target_arch = "wasm32")]
use std::ops::{Deref, DerefMut};

#[cfg(target_arch = "wasm32")]
impl MmapMut {

	pub unsafe fn map_mut(_file: &File) -> Result<MmapMut> {
		Err(ErrorKind::Other.into())
	}
 
	pub fn flush(&self) -> Result<()> {
		Err(ErrorKind::Other.into())
	}
}


#[cfg(target_arch = "wasm32")]
impl Deref for MmapMut {
	type Target = [u8];

	#[inline]
	fn deref(&self) -> &[u8] {
		unimplemented!()
	}
}

#[cfg(target_arch = "wasm32")]
impl DerefMut for MmapMut {
	#[inline]
	fn deref_mut(&mut self) -> &mut [u8] {
		unimplemented!()
	}
}

#[cfg(target_arch = "wasm32")]
impl AsRef<[u8]> for MmapMut {
	#[inline]
	fn as_ref(&self) -> &[u8] {
		unimplemented!()
	}
}

