// Copyright 2015-2018 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Primitive types shared by Substrate and Parity Ethereum.
//!
//! Those are uint types `U256` and `U512`, and fixed hash types `H160`,
//! `H256` and `H512`, with optional serde serialization, parity-codec and
//! rlp encoding.

#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate uint;

#[macro_use]
extern crate fixed_hash;

#[macro_use]
extern crate crunchy;

#[cfg(feature = "impl-serde")]
#[macro_use]
extern crate impl_serde;

#[cfg(feature = "impl-codec")]
#[macro_use]
extern crate impl_codec;

#[cfg(feature = "impl-rlp")]
#[macro_use]
extern crate impl_rlp;

construct_uint!(U256, 4);
#[cfg(feature = "impl-serde")]
impl_uint_serde!(U256, 4);
#[cfg(feature = "impl-codec")]
impl_uint_codec!(U256, 4);
#[cfg(feature = "impl-rlp")]
impl_uint_rlp!(U256, 4);

impl U256 {
	/// Multiplies two 256-bit integers to produce full 512-bit integer
	/// No overflow possible
	#[inline(always)]
	pub fn full_mul(self, other: U256) -> U512 {
		U512(uint_full_mul_reg!(U256, 4, self, other))
	}
}

impl From<U256> for U512 {
	fn from(value: U256) -> U512 {
		let U256(ref arr) = value;
		let mut ret = [0; 8];
		ret[0] = arr[0];
		ret[1] = arr[1];
		ret[2] = arr[2];
		ret[3] = arr[3];
		U512(ret)
	}
}

impl From<U512> for U256 {
	fn from(value: U512) -> U256 {
		let U512(ref arr) = value;
		if arr[4] | arr[5] | arr[6] | arr[7] != 0 {
			panic!("From<U512> for U256: encountered overflow")
		}
		let mut ret = [0; 4];
		ret[0] = arr[0];
		ret[1] = arr[1];
		ret[2] = arr[2];
		ret[3] = arr[3];
		U256(ret)
	}
}

impl<'a> From<&'a U256> for U512 {
	fn from(value: &'a U256) -> U512 {
		let U256(ref arr) = *value;
		let mut ret = [0; 8];
		ret[0] = arr[0];
		ret[1] = arr[1];
		ret[2] = arr[2];
		ret[3] = arr[3];
		U512(ret)
	}
}

impl<'a> From<&'a U512> for U256 {
	fn from(value: &'a U512) -> U256 {
		let U512(ref arr) = *value;
		if arr[4] | arr[5] | arr[6] | arr[7] != 0 {
			panic!("From<&U512> for U256: encountered overflow")
		}
		let mut ret = [0; 4];
		ret[0] = arr[0];
		ret[1] = arr[1];
		ret[2] = arr[2];
		ret[3] = arr[3];
		U256(ret)
	}
}

construct_uint!(U512, 8);
#[cfg(feature = "impl-serde")]
impl_uint_serde!(U512, 8);
#[cfg(feature = "impl-codec")]
impl_uint_codec!(U512, 8);
#[cfg(feature = "impl-rlp")]
impl_uint_rlp!(U512, 8);

construct_fixed_hash! {
	/// Fixed-size uninterpreted hash type with 20 bytes (160 bits) size.
	pub struct H160(20);
}
#[cfg(feature = "impl-serde")]
impl_fixed_hash_serde!(H160, 20);
#[cfg(feature = "impl-codec")]
impl_fixed_hash_codec!(H160, 20);
#[cfg(feature = "impl-rlp")]
impl_fixed_hash_rlp!(H160, 20);

impl From<H160> for H256 {
	fn from(value: H160) -> H256 {
		let mut ret = H256::zero();
		ret.0[12..32].copy_from_slice(value.as_bytes());
		ret
	}
}

impl<'a> From<&'a H160> for H256 {
	fn from(value: &'a H160) -> H256 {
		let mut ret = H256::zero();
		ret.0[12..32].copy_from_slice(value.as_bytes());
		ret
	}
}

construct_fixed_hash! {
	/// Fixed-size uninterpreted hash type with 32 bytes (256 bits) size.
	pub struct H256(32);
}
#[cfg(feature = "impl-serde")]
impl_fixed_hash_serde!(H256, 32);
#[cfg(feature = "impl-codec")]
impl_fixed_hash_codec!(H256, 32);
#[cfg(feature = "impl-rlp")]
impl_fixed_hash_rlp!(H256, 32);

construct_fixed_hash! {
	/// Fixed-size uninterpreted hash type with 64 bytes (512 bits) size.
	pub struct H512(64);
}
#[cfg(feature = "impl-serde")]
impl_fixed_hash_serde!(H512, 64);
#[cfg(feature = "impl-codec")]
impl_fixed_hash_codec!(H512, 64);
#[cfg(feature = "impl-rlp")]
impl_fixed_hash_rlp!(H512, 64);
