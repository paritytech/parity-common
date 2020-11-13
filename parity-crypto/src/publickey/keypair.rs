// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Key pair (public + secret) description.

use super::{Address, Error, Public, Secret};
use crate::Keccak256;
use secp256k1::{key, SECP256K1};
use std::fmt;

/// Convert public key into the address
pub fn public_to_address(public: &Public) -> Address {
	let hash = public.keccak256();
	let mut result = Address::zero();
	result.as_bytes_mut().copy_from_slice(&hash[12..]);
	result
}

#[derive(Debug, Clone, PartialEq)]
/// secp256k1 key pair
pub struct KeyPair {
	secret: Secret,
	public: Public,
}

impl fmt::Display for KeyPair {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		writeln!(f, "secret:  {:x}", self.secret)?;
		writeln!(f, "public:  {:x}", self.public)?;
		write!(f, "address: {:x}", self.address())
	}
}

impl KeyPair {
	/// Create a pair from secret key
	pub fn from_secret(secret: Secret) -> Result<KeyPair, Error> {
		let context = &SECP256K1;
		let s: key::SecretKey = key::SecretKey::from_slice(&secret[..])?;
		let pub_key = key::PublicKey::from_secret_key(context, &s);
		let serialized = pub_key.serialize_uncompressed();

		let mut public = Public::default();
		public.as_bytes_mut().copy_from_slice(&serialized[1..65]);

		let keypair = KeyPair { secret, public };

		Ok(keypair)
	}

	/// Create a pair from the slice, which imported and verified as secret key
	pub fn from_secret_slice(slice: &[u8]) -> Result<KeyPair, Error> {
		Self::from_secret(Secret::import_key(slice)?)
	}

	/// Copies a pair from another one
	#[inline(always)]
	pub fn from_keypair(sec: key::SecretKey, publ: key::PublicKey) -> Self {
		let serialized = publ.serialize_uncompressed();
		let secret = Secret::from(sec);
		let mut public = Public::default();
		public.as_bytes_mut().copy_from_slice(&serialized[1..65]);

		KeyPair { secret, public }
	}

	/// Returns secret part of the keypair
	pub fn secret(&self) -> &Secret {
		&self.secret
	}

	/// Returns public part of the keypair
	pub fn public(&self) -> &Public {
		&self.public
	}

	/// Returns public part of the keypair converted into Address
	pub fn address(&self) -> Address {
		public_to_address(&self.public)
	}
}

#[cfg(test)]
mod tests {
	use super::{KeyPair, Secret};

	#[test]
	fn from_secret() {
		let secret =
			Secret::copy_from_str(&"a100df7a048e50ed308ea696dc600215098141cb391e9527329df289f9383f65").unwrap();
		let _ = KeyPair::from_secret(secret).unwrap();
	}

	#[test]
	fn keypair_display() {
		let expected =
"secret:  a100df7a048e50ed308ea696dc600215098141cb391e9527329df289f9383f65
public:  8ce0db0b0359ffc5866ba61903cc2518c3675ef2cf380a7e54bde7ea20e6fa1ab45b7617346cd11b7610001ee6ae5b0155c41cad9527cbcdff44ec67848943a4
address: 5b073e9233944b5e729e46d618f0d8edf3d9c34a".to_owned();
		let secret =
			Secret::copy_from_str(&"a100df7a048e50ed308ea696dc600215098141cb391e9527329df289f9383f65").unwrap();
		let kp = KeyPair::from_secret(secret).unwrap();
		assert_eq!(format!("{}", kp), expected);
	}
}
