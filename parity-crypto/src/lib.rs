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

//! Crypto utils used by ethstore and network.

#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate lazy_static;
#[cfg(not(target_arch = "wasm32"))]
extern crate ring;
#[cfg(target_arch = "wasm32")]
extern crate subtle;
extern crate tiny_keccak;
extern crate scrypt as rscrypt;
extern crate ripemd160 as rripemd160;
extern crate sha2 as rsha2;
extern crate digest as rdigest;
extern crate aes as raes;
extern crate aes_ctr;
extern crate block_modes;


pub mod aes;
#[cfg(not(target_arch = "wasm32"))]
pub mod aes_gcm;
// could create a less safe RustCrypto based aes_gcm here if needed for wasm
pub mod error;
pub mod scrypt;
pub mod digest;
#[cfg(not(target_arch = "wasm32"))]
pub mod hmac;
#[cfg(all(not(target_arch = "wasm32"), test))]
pub mod hmac_alt;
#[path = "hmac_alt.rs"]
#[cfg(target_arch = "wasm32")]
pub mod hmac;


#[cfg(not(target_arch = "wasm32"))]
pub mod secp256k1;

#[cfg(all(not(target_arch = "wasm32"), test))]
pub mod secp256k1_alt;
#[path = "secp256k1_alt.rs"]
#[cfg(target_arch = "wasm32")]
pub mod secp256k1;

// could create a less safe crate using RustCrypto or just switch
#[cfg(not(target_arch = "wasm32"))]
pub mod pbkdf2;
// could create a less safe crate using RustCrypto or just switch

pub use error::Error;

use tiny_keccak::Keccak;

pub const KEY_LENGTH: usize = 32;
pub const KEY_ITERATIONS: usize = 10240;
pub const KEY_LENGTH_AES: usize = KEY_LENGTH / 2;

/// Default authenticated data to use (in RPC).
pub const DEFAULT_MAC: [u8; 2] = [0, 0];

pub trait Keccak256<T> {
	fn keccak256(&self) -> T where T: Sized;
}

impl<T> Keccak256<[u8; 32]> for T where T: AsRef<[u8]> {
	fn keccak256(&self) -> [u8; 32] {
		let mut keccak = Keccak::new_keccak256();
		let mut result = [0u8; 32];
		keccak.update(self.as_ref());
		keccak.finalize(&mut result);
		result
	}
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
pub fn is_equal(a: &[u8], b: &[u8]) -> bool {
	ring::constant_time::verify_slices_are_equal(a, b).is_ok()
}

#[cfg(target_arch = "wasm32")]
pub fn is_equal(a: &[u8], b: &[u8]) -> bool {
	use subtle::ConstantTimeEq;
	a.ct_eq(b).unwrap_u8() == 1
}


