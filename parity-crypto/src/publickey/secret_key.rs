// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Secret key implementation.

use std::convert::TryFrom;
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use ethereum_types::H256;
use secp256k1::constants::SECRET_KEY_SIZE as SECP256K1_SECRET_KEY_SIZE;
use secp256k1::key;
use zeroize::Zeroize;

use crate::publickey::Error;

/// Represents secret key
#[derive(Clone, PartialEq, Eq)]
pub struct Secret {
	inner: H256,
}

impl Drop for Secret {
	fn drop(&mut self) {
		self.inner.0.zeroize()
	}
}

impl fmt::LowerHex for Secret {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.inner.fmt(fmt)
	}
}

impl fmt::Debug for Secret {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.inner.fmt(fmt)
	}
}

impl fmt::Display for Secret {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "Secret: 0x{:x}{:x}..{:x}{:x}", self.inner[0], self.inner[1], self.inner[30], self.inner[31])
	}
}

impl Secret {
	/// Creates a `Secret` from the given slice, returning `None` if the slice length != 32.
	pub fn copy_from_slice(key: &[u8]) -> Option<Self> {
		if key.len() != 32 {
			return None;
		}
		let mut h = H256::zero();
		h.as_bytes_mut().copy_from_slice(&key[0..32]);
		Some(Secret { inner: h })
	}

	/// Creates zero key, which is invalid for crypto operations, but valid for math operation.
	pub fn zero() -> Self {
		Secret { inner: H256::zero() }
	}

	/// Imports and validates the key.
	pub fn import_key(key: &[u8]) -> Result<Self, Error> {
		let secret = key::SecretKey::from_slice(key)?;
		Ok(secret.into())
	}

	/// Checks validity of this key.
	pub fn check_validity(&self) -> Result<(), Error> {
		self.to_secp256k1_secret().map(|_| ())
	}

	/// Wrapper over hex conversion
	pub fn to_hex(&self) -> String {
		format!("{:x}", self.inner)
	}

	/// Inplace add one secret key to another (scalar + scalar)
	pub fn add(&mut self, other: &Secret) -> Result<(), Error> {
		match (self.is_zero(), other.is_zero()) {
			(true, true) | (false, true) => Ok(()),
			(true, false) => {
				*self = other.clone();
				Ok(())
			}
			(false, false) => {
				let mut key_secret = self.to_secp256k1_secret()?;
				let other_secret = other.to_secp256k1_secret()?;
				key_secret.add_assign(&other_secret[..])?;

				*self = key_secret.into();
				Ok(())
			}
		}
	}

	/// Inplace subtract one secret key from another (scalar - scalar)
	pub fn sub(&mut self, other: &Secret) -> Result<(), Error> {
		match (self.is_zero(), other.is_zero()) {
			(true, true) | (false, true) => Ok(()),
			(true, false) => {
				*self = other.clone();
				self.neg()
			}
			(false, false) => {
				let mut key_secret = self.to_secp256k1_secret()?;
				let mut other_secret = other.to_secp256k1_secret()?;
				other_secret.mul_assign(super::MINUS_ONE_KEY)?;
				key_secret.add_assign(&other_secret[..])?;

				*self = key_secret.into();
				Ok(())
			}
		}
	}

	/// Inplace decrease secret key (scalar - 1)
	pub fn dec(&mut self) -> Result<(), Error> {
		match self.is_zero() {
			true => {
				*self = Secret::try_from(super::MINUS_ONE_KEY)
					.expect("Constructing a secret key from a known-good constant works; qed.");
				Ok(())
			}
			false => {
				let mut key_secret = self.to_secp256k1_secret()?;
				key_secret.add_assign(super::MINUS_ONE_KEY)?;

				*self = key_secret.into();
				Ok(())
			}
		}
	}

	/// Inplace multiply one secret key to another (scalar * scalar)
	pub fn mul(&mut self, other: &Secret) -> Result<(), Error> {
		match (self.is_zero(), other.is_zero()) {
			(true, true) | (true, false) => Ok(()),
			(false, true) => {
				*self = Self::zero();
				Ok(())
			}
			(false, false) => {
				let mut key_secret = self.to_secp256k1_secret()?;
				let other_secret = other.to_secp256k1_secret()?;
				key_secret.mul_assign(&other_secret[..])?;

				*self = key_secret.into();
				Ok(())
			}
		}
	}

	/// Inplace negate secret key (-scalar)
	pub fn neg(&mut self) -> Result<(), Error> {
		match self.is_zero() {
			true => Ok(()),
			false => {
				let mut key_secret = self.to_secp256k1_secret()?;
				key_secret.mul_assign(super::MINUS_ONE_KEY)?;

				*self = key_secret.into();
				Ok(())
			}
		}
	}

	/// Compute power of secret key inplace (secret ^ pow).
	pub fn pow(&mut self, pow: usize) -> Result<(), Error> {
		if self.is_zero() {
			return Ok(());
		}

		match pow {
			0 => *self = key::ONE_KEY.into(),
			1 => (),
			_ => {
				let c = self.clone();
				for _ in 1..pow {
					self.mul(&c)?;
				}
			}
		}

		Ok(())
	}

	/// Create a `secp256k1::key::SecretKey` based on this secret.
	pub fn to_secp256k1_secret(&self) -> Result<key::SecretKey, Error> {
		key::SecretKey::from_slice(&self[..]).map_err(Into::into)
	}
}

impl FromStr for Secret {
	type Err = Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(H256::from_str(s).map_err(|e| Error::Custom(format!("{:?}", e)))?.into())
	}
}

impl From<[u8; 32]> for Secret {
	fn from(k: [u8; 32]) -> Self {
		Secret { inner: H256(k) }
	}
}

impl From<H256> for Secret {
	fn from(s: H256) -> Self {
		s.0.into()
	}
}

impl TryFrom<&str> for Secret {
	type Error = Error;

	fn try_from(s: &str) -> Result<Self, Error> {
		s.parse().map_err(|e| Error::Custom(format!("{:?}", e)))
	}
}

impl TryFrom<&[u8]> for Secret {
	type Error = Error;

	fn try_from(b: &[u8]) -> Result<Self, Error> {
		if b.len() != SECP256K1_SECRET_KEY_SIZE {
			return Err(Error::InvalidSecretKey);
		}
		Ok(Self { inner: H256::from_slice(b) })
	}
}

impl From<key::SecretKey> for Secret {
	fn from(key: key::SecretKey) -> Self {
		let mut a = [0; SECP256K1_SECRET_KEY_SIZE];
		a.copy_from_slice(&key[0..SECP256K1_SECRET_KEY_SIZE]);
		a.into()
	}
}

impl Deref for Secret {
	type Target = H256;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[cfg(test)]
mod tests {
	use super::super::{Generator, Random};
	use super::Secret;
	use std::str::FromStr;

	#[test]
	fn secret_pow() {
		let secret = Random.generate().secret().clone();

		let mut pow0 = secret.clone();
		pow0.pow(0).unwrap();
		assert_eq!(pow0, Secret::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap());

		let mut pow1 = secret.clone();
		pow1.pow(1).unwrap();
		assert_eq!(pow1, secret);

		let mut pow2 = secret.clone();
		pow2.pow(2).unwrap();
		let mut pow2_expected = secret.clone();
		pow2_expected.mul(&secret).unwrap();
		assert_eq!(pow2, pow2_expected);

		let mut pow3 = secret.clone();
		pow3.pow(3).unwrap();
		let mut pow3_expected = secret.clone();
		pow3_expected.mul(&secret).unwrap();
		pow3_expected.mul(&secret).unwrap();
		assert_eq!(pow3, pow3_expected);
	}
}
