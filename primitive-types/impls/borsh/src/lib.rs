// Copyright 2022 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Serde serialization support for uint and fixed hash.

#![no_std]

#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[doc(hidden)]
pub use borsh;

/// Add Borsh serialization support to an integer created by `construct_uint!`.
#[macro_export]
macro_rules! impl_uint_borsh {
    ($name: ident, $len: expr) => {
        impl $crate::borsh::BorshSerialize for $name {
            fn serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
                self.0.serialize(writer)
            }
        }

        impl $crate::borsh::BorshDeserialize for $name {
            fn deserialize(buf: &mut &[u8]) -> Result<Self, std::io::Error> {
                <[u64; $len]>::deserialize(buf).map(|bytes| Self(bytes))
            }
        }
    };
}

/// Add Borsh serialization support to a fixed-sized hash type created by `construct_fixed_hash!`.
#[macro_export]
macro_rules! impl_fixed_hash_borsh {
    ($name: ident, $len: expr) => {
        impl $crate::borsh::BorshSerialize for $name {
            fn serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
                self.0.serialize(writer)
            }
        }

        impl $crate::borsh::BorshDeserialize for $name {
            fn deserialize(buf: &mut &[u8]) -> Result<Self, std::io::Error> {
                <[u8; $len]>::deserialize(buf).map(|bytes| Self(bytes))
            }
        }
    };
}
