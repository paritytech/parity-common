// Copyright 2015-2017 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature="libc", not(target_os = "unknown")))]
#[doc(hidden)]
pub extern crate libc;

#[cfg(not(feature="libc"))]
#[doc(hidden)]
pub mod libc { }

#[cfg(feature="heapsizeof")]
#[doc(hidden)]
pub extern crate heapsize;

#[cfg(feature="std")]
#[doc(hidden)]
pub extern crate core;

#[cfg(feature="std")]
#[doc(hidden)]
pub extern crate rustc_hex;

#[cfg(feature="std")]
#[doc(hidden)]
pub extern crate rand;

#[cfg(feature="impl_quickcheck_arbitrary")]
#[doc(hidden)]
pub extern crate quickcheck;

#[cfg(feature="uint_conversions")]
extern crate uint;

#[cfg(feature="serialize")]
extern crate serde;

#[cfg(feature="serialize")]
#[macro_use]
extern crate serde_derive;

mod hash;
pub use hash::*;
