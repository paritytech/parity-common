// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Crypto utils used by ethstore and network.

pub mod aes;
pub mod digest;
pub mod error;
pub mod hmac;
pub mod pbkdf2;
#[cfg(feature = "publickey")]
pub mod publickey;
pub mod scrypt;

pub use crate::error::Error;

use subtle::ConstantTimeEq;
use tiny_keccak::{Hasher, Keccak};

pub const KEY_LENGTH: usize = 32;
pub const KEY_ITERATIONS: usize = 10240;
pub const KEY_LENGTH_AES: usize = KEY_LENGTH / 2;

/// Default authenticated data to use (in RPC).
pub const DEFAULT_MAC: [u8; 2] = [0, 0];

pub trait Keccak256<T> {
	fn keccak256(&self) -> T
	where
		T: Sized;
}

impl<T> Keccak256<[u8; 32]> for T
where
	T: AsRef<[u8]>,
{
	fn keccak256(&self) -> [u8; 32] {
		let mut keccak = Keccak::v256();
		let mut result = [0u8; 32];
		keccak.update(self.as_ref());
		keccak.finalize(&mut result);
		result
	}
}

pub fn derive_key_iterations(password: &[u8], salt: &[u8], c: u32) -> (Vec<u8>, Vec<u8>) {
	let mut derived_key = [0u8; KEY_LENGTH];
	pbkdf2::sha256(c, pbkdf2::Salt(salt), pbkdf2::Secret(password), &mut derived_key);
	let derived_right_bits = &derived_key[0..KEY_LENGTH_AES];
	let derived_left_bits = &derived_key[KEY_LENGTH_AES..KEY_LENGTH];
	(derived_right_bits.to_vec(), derived_left_bits.to_vec())
}

pub fn derive_mac(derived_left_bits: &[u8], cipher_text: &[u8]) -> Vec<u8> {
	let mut mac = vec![0u8; KEY_LENGTH_AES + cipher_text.len()];
	mac[0..KEY_LENGTH_AES].copy_from_slice(derived_left_bits);
	mac[KEY_LENGTH_AES..cipher_text.len() + KEY_LENGTH_AES].copy_from_slice(cipher_text);
	mac
}

pub fn is_equal(a: &[u8], b: &[u8]) -> bool {
	a.ct_eq(b).into()
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn can_test_for_equality() {
		let a = b"abc";
		let b = b"abc";
		let c = b"efg";
		assert!(is_equal(a, b));
		assert!(!is_equal(a, c));
	}
}
