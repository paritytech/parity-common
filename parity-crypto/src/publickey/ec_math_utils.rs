// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Multiple primitives for work with public and secret keys and with secp256k1 curve points

use super::{Error, Public, Secret};
use ethereum_types::{BigEndianHash as _, H256, U256};
use lazy_static::lazy_static;
use secp256k1::constants::CURVE_ORDER as SECP256K1_CURVE_ORDER;
use secp256k1::key;
use secp256k1::SECP256K1;

/// Generation point array combined from X and Y coordinates
/// Equivalent to uncompressed form, see https://tools.ietf.org/id/draft-jivsov-ecc-compact-05.html#rfc.section.3
pub const BASE_POINT_BYTES: [u8; 65] = [
	0x4, // The X coordinate of the generator
	0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce, 0x87, 0x0b, 0x07, 0x02, 0x9b, 0xfc,
	0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8, 0x17, 0x98,
	// The Y coordinate of the generator
	0x48, 0x3a, 0xda, 0x77, 0x26, 0xa3, 0xc4, 0x65, 0x5d, 0xa4, 0xfb, 0xfc, 0x0e, 0x11, 0x08, 0xa8, 0xfd, 0x17, 0xb4,
	0x48, 0xa6, 0x85, 0x54, 0x19, 0x9c, 0x47, 0xd0, 0x8f, 0xfb, 0x10, 0xd4, 0xb8,
];

lazy_static! {
	pub static ref CURVE_ORDER: U256 = H256::from_slice(&SECP256K1_CURVE_ORDER).into_uint();
}

/// In-place multiply public key by secret key (EC point * scalar)
pub fn public_mul_secret(public: &mut Public, secret: &Secret) -> Result<(), Error> {
	let key_secret = secret.to_secp256k1_secret()?;
	let mut key_public = to_secp256k1_public(public)?;
	key_public.mul_assign(&SECP256K1, &key_secret[..])?;
	set_public(public, &key_public);
	Ok(())
}

/// In-place add one public key to another (EC point + EC point)
pub fn public_add(public: &mut Public, other: &Public) -> Result<(), Error> {
	let key_public = to_secp256k1_public(public)?;
	let other_public = to_secp256k1_public(other)?;
	let key_public = key_public.combine(&other_public)?;
	set_public(public, &key_public);
	Ok(())
}

/// In-place sub one public key from another (EC point - EC point)
pub fn public_sub(public: &mut Public, other: &Public) -> Result<(), Error> {
	let mut key_neg_other = to_secp256k1_public(other)?;
	key_neg_other.mul_assign(&SECP256K1, super::MINUS_ONE_KEY)?;

	let mut key_public = to_secp256k1_public(public)?;
	key_public = key_public.combine(&key_neg_other)?;
	set_public(public, &key_public);
	Ok(())
}

/// Replace a public key with its additive inverse (EC point = - EC point)
pub fn public_negate(public: &mut Public) -> Result<(), Error> {
	let mut key_public = to_secp256k1_public(public)?;
	key_public.mul_assign(&SECP256K1, super::MINUS_ONE_KEY)?;
	set_public(public, &key_public);
	Ok(())
}

/// Return the generation point (aka base point) of secp256k1
pub fn generation_point() -> Public {
	let public_key = key::PublicKey::from_slice(&BASE_POINT_BYTES).expect("constructed using constants; qed");
	let mut public = Public::default();
	set_public(&mut public, &public_key);
	public
}

fn to_secp256k1_public(public: &Public) -> Result<key::PublicKey, Error> {
	let public_data = {
		let mut temp = [4u8; 65];
		(&mut temp[1..65]).copy_from_slice(&public[0..64]);
		temp
	};

	Ok(key::PublicKey::from_slice(&public_data)?)
}

fn set_public(public: &mut Public, key_public: &key::PublicKey) {
	let key_public_serialized = key_public.serialize_uncompressed();
	public.as_bytes_mut().copy_from_slice(&key_public_serialized[1..65]);
}

#[cfg(test)]
mod tests {
	use super::super::{Generator, Random, Secret};
	use super::{generation_point, public_add, public_mul_secret, public_negate, public_sub};

	#[test]
	fn public_addition_is_commutative() {
		let public1 = Random.generate().public().clone();
		let public2 = Random.generate().public().clone();

		let mut left = public1.clone();
		public_add(&mut left, &public2).unwrap();

		let mut right = public2.clone();
		public_add(&mut right, &public1).unwrap();

		assert_eq!(left, right);
	}

	#[test]
	fn public_addition_is_reversible_with_subtraction() {
		let public1 = Random.generate().public().clone();
		let public2 = Random.generate().public().clone();

		let mut sum = public1.clone();
		public_add(&mut sum, &public2).unwrap();
		public_sub(&mut sum, &public2).unwrap();

		assert_eq!(sum, public1);
	}

	#[test]
	fn public_negation_is_involutory() {
		let public = Random.generate().public().clone();
		let mut negation = public.clone();
		public_negate(&mut negation).unwrap();
		public_negate(&mut negation).unwrap();

		assert_eq!(negation, public);
	}

	#[test]
	fn generation_point_expected() {
		let point = generation_point();
		// Check the returned value equal to uncompressed form for sec2561k1
		assert_eq!(format!("{:x}", point), "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8");
	}

	#[test]
	fn public_multiplication_verification() {
		let secret =
			Secret::copy_from_str(&"a100df7a048e50ed308ea696dc600215098141cb391e9527329df289f9383f65").unwrap();
		let mut public = generation_point();
		public_mul_secret(&mut public, &secret).unwrap();
		assert_eq!(format!("{:x}", public), "8ce0db0b0359ffc5866ba61903cc2518c3675ef2cf380a7e54bde7ea20e6fa1ab45b7617346cd11b7610001ee6ae5b0155c41cad9527cbcdff44ec67848943a4");
	}
}
