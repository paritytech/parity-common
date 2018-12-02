// Copyright 2015-2017 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Efficient large, fixed-size big integers and hashes.

#![cfg_attr(not(feature = "std"), no_std)]

#[doc(hidden)]
pub extern crate byteorder;

#[cfg(feature="heapsize")]
#[doc(hidden)]
pub extern crate heapsize;

// Re-export libcore using an alias so that the macros can work without
// requiring `extern crate core` downstream.
#[doc(hidden)]
pub extern crate core as core_;

#[doc(hidden)]
pub extern crate rustc_hex;

#[cfg(feature="quickcheck")]
#[doc(hidden)]
pub extern crate quickcheck;

extern crate crunchy;
pub use crunchy::unroll;

#[macro_use]
mod uint;
pub use uint::*;

#[cfg(feature = "common")]
mod common {
	construct_uint! {
		/// Little-endian 256-bit integer type.
		#[derive(Copy, Clone, Eq, PartialEq, Hash)]
		pub struct U256(4);
	}

	construct_uint! {
		/// Little-endian 512-bit integer type.
		#[derive(Copy, Clone, Eq, PartialEq, Hash)]
		pub struct U512(8);
	}

	#[doc(hidden)]
	impl U256 {
		/// Multiplies two 256-bit integers to produce full 512-bit integer
		/// No overflow possible
		#[inline(always)]
		pub fn full_mul(self, other: U256) -> U512 {
			U512(uint_full_mul_reg!(U256, 4, self, other))
		}
	}
}

#[cfg(feature = "common")]
pub use common::{U256, U512};
