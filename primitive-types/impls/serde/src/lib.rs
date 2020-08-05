// Copyright 2020 Parity Technologies
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
pub use serde;

#[doc(hidden)]
pub mod serialize;

/// Add Serde serialization support to an integer created by `construct_uint!`.
#[macro_export]
macro_rules! impl_uint_serde {
	($name: ident, $len: expr) => {
		impl $crate::serde::Serialize for $name {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
			where
				S: $crate::serde::Serializer,
			{
				let mut slice = [0u8; 2 + 2 * $len * 8];
				let mut bytes = [0u8; $len * 8];
				self.to_big_endian(&mut bytes);
				$crate::serialize::serialize_uint(&mut slice, &bytes, serializer)
			}
		}

		impl<'de> $crate::serde::Deserialize<'de> for $name {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: $crate::serde::Deserializer<'de>,
			{
				let mut bytes = [0u8; $len * 8];
				let wrote = $crate::serialize::deserialize_check_len(
					deserializer,
					$crate::serialize::ExpectedLen::Between(0, &mut bytes),
				)?;
				Ok(bytes[0..wrote].into())
			}
		}
	};
}

/// Add Serde serialization support to a fixed-sized hash type created by `construct_fixed_hash!`.
#[macro_export]
macro_rules! impl_fixed_hash_serde {
	($name: ident, $len: expr) => {
		impl $crate::serde::Serialize for $name {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
			where
				S: $crate::serde::Serializer,
			{
				let mut slice = [0u8; 2 + 2 * $len];
				$crate::serialize::serialize_raw(&mut slice, &self.0, serializer)
			}
		}

		impl<'de> $crate::serde::Deserialize<'de> for $name {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: $crate::serde::Deserializer<'de>,
			{
				let mut bytes = [0u8; $len];
				$crate::serialize::deserialize_check_len(
					deserializer,
					$crate::serialize::ExpectedLen::Exact(&mut bytes),
				)?;
				Ok($name(bytes))
			}
		}
	};
}
