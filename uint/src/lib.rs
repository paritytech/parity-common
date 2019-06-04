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
pub use crate::uint::*;
