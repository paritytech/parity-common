// Copyright 2015-2017 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(not(feature = "std"), no_std)]

// Re-export libcore using an alias so that the macros can work without
// requiring `extern crate core` downstream.
#[doc(hidden)]
pub extern crate core as core_;

#[cfg(all(feature = "libc", not(target_os = "unknown")))]
#[doc(hidden)]
pub extern crate libc;

#[macro_use(const_assert)]
#[allow(unused)]
// This disables a warning for unused #[macro_use(..)]
// which is incorrect since the compiler does not check
// for all available configurations.
#[doc(hidden)]
pub extern crate static_assertions;

// Export `const_assert` macro so that users of this crate do not
// have to import the `static_assertions` crate themselves.
#[doc(hidden)]
pub use static_assertions::const_assert;

#[cfg(feature = "byteorder")]
#[doc(hidden)]
pub extern crate byteorder;

#[cfg(not(feature = "libc"))]
#[doc(hidden)]
pub mod libc {}

#[cfg(feature = "heapsize")]
#[doc(hidden)]
pub extern crate heapsize;

#[cfg(feature = "rustc-hex")]
#[doc(hidden)]
pub extern crate rustc_hex;

#[cfg(feature = "rand")]
#[doc(hidden)]
pub extern crate rand;

#[cfg(feature = "quickcheck")]
#[doc(hidden)]
pub extern crate quickcheck;

#[macro_use]
mod hash;

#[cfg(test)]
mod tests;

#[cfg(feature = "api-dummy")]
construct_fixed_hash! {
    /// Go here for an overview of the hash type API.
    pub struct ApiDummy(32);
}
