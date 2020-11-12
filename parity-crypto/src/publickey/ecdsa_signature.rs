// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Signature based on ECDSA, algorithm's description: https://en.wikipedia.org/wiki/Elliptic_Curve_Digital_Signature_Algorithm

use super::{public_to_address, Address, Error, Message, Public, Secret};
use ethereum_types::{H256, H520};
use rustc_hex::{FromHex, ToHex};
use secp256k1::{
	key::{PublicKey, SecretKey},
	recovery::{RecoverableSignature, RecoveryId},
	Error as SecpError, Message as SecpMessage, SECP256K1,
};
use std::{
	cmp::PartialEq,
	fmt,
	hash::{Hash, Hasher},
	ops::{Deref, DerefMut},
	str::FromStr,
};

/// Signature encoded as RSV components
#[repr(C)]
pub struct Signature([u8; 65]);

impl Signature {
	/// Get a slice into the 'r' portion of the data.
	pub fn r(&self) -> &[u8] {
		&self.0[0..32]
	}

	/// Get a slice into the 's' portion of the data.
	pub fn s(&self) -> &[u8] {
		&self.0[32..64]
	}

	/// Get the recovery byte.
	pub fn v(&self) -> u8 {
		self.0[64]
	}

	/// Encode the signature into RSV array (V altered to be in "Electrum" notation).
	pub fn into_electrum(mut self) -> [u8; 65] {
		self.0[64] += 27;
		self.0
	}

	/// Parse bytes as a signature encoded as RSV (V in "Electrum" notation).
	/// May return empty (invalid) signature if given data has invalid length.
	pub fn from_electrum(data: &[u8]) -> Self {
		if data.len() != 65 || data[64] < 27 {
			// fallback to empty (invalid) signature
			return Signature::default();
		}

		let mut sig = [0u8; 65];
		sig.copy_from_slice(data);
		sig[64] -= 27;
		Signature(sig)
	}

	/// Create a signature object from the RSV triple.
	pub fn from_rsv(r: &H256, s: &H256, v: u8) -> Self {
		let mut sig = [0u8; 65];
		sig[0..32].copy_from_slice(r.as_ref());
		sig[32..64].copy_from_slice(s.as_ref());
		sig[64] = v;
		Signature(sig)
	}

	/// Check if this is a "low" signature (that s part of the signature is in range
	/// 0x1 and 0x7FFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF 5D576E73 57A4501D DFE92F46 681B20A0 (inclusive)).
	/// This condition may be required by some verification algorithms
	pub fn is_low_s(&self) -> bool {
		const LOW_SIG_THRESHOLD: H256 = H256([
			0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x5D, 0x57,
			0x6E, 0x73, 0x57, 0xA4, 0x50, 0x1D, 0xDF, 0xE9, 0x2F, 0x46, 0x68, 0x1B, 0x20, 0xA0,
		]);
		H256::from_slice(self.s()) <= LOW_SIG_THRESHOLD
	}

	/// Check if each component of the signature is in valid range.
	/// r is in range 0x1 and 0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141 (inclusive)
	/// s is in range 0x1 and fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141 (inclusive)
	/// v is 0 or 1
	/// Group order for secp256k1 defined as 'n' in "Standards for Efficient Cryptography" (SEC2) 2.7.1;
	/// used here as the upper bound for a valid (r, s, v) tuple
	pub fn is_valid(&self) -> bool {
		const UPPER_BOUND: H256 = H256([
			0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0xba, 0xae,
			0xdc, 0xe6, 0xaf, 0x48, 0xa0, 0x3b, 0xbf, 0xd2, 0x5e, 0x8c, 0xd0, 0x36, 0x41, 0x41,
		]);
		const ONE: H256 = H256([
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
		]);
		let r = H256::from_slice(self.r());
		let s = H256::from_slice(self.s());
		self.v() <= 1 && r < UPPER_BOUND && r >= ONE && s < UPPER_BOUND && s >= ONE
	}
}

// manual implementation large arrays don't have trait impls by default.
// TODO[grbIzl] remove when integer generics exist
impl PartialEq for Signature {
	fn eq(&self, other: &Self) -> bool {
		&self.0[..] == &other.0[..]
	}
}

// manual implementation required in Rust 1.13+, see `std::cmp::AssertParamIsEq`.
impl Eq for Signature {}

// also manual for the same reason, but the pretty printing might be useful.
impl fmt::Debug for Signature {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		f.debug_struct("Signature")
			.field("r", &self.0[0..32].to_hex::<String>())
			.field("s", &self.0[32..64].to_hex::<String>())
			.field("v", &self.0[64..65].to_hex::<String>())
			.finish()
	}
}

impl fmt::Display for Signature {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "{}", self.to_hex::<String>())
	}
}

impl FromStr for Signature {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.from_hex::<Vec<u8>>() {
			Ok(ref hex) if hex.len() == 65 => {
				let mut data = [0; 65];
				data.copy_from_slice(&hex[0..65]);
				Ok(Signature(data))
			}
			_ => Err(Error::InvalidSignature),
		}
	}
}

impl Default for Signature {
	fn default() -> Self {
		Signature([0; 65])
	}
}

impl Hash for Signature {
	fn hash<H: Hasher>(&self, state: &mut H) {
		H520::from(self.0).hash(state);
	}
}

impl Clone for Signature {
	fn clone(&self) -> Self {
		Signature(self.0.clone())
	}
}

impl From<[u8; 65]> for Signature {
	fn from(s: [u8; 65]) -> Self {
		Signature(s)
	}
}

impl Into<[u8; 65]> for Signature {
	fn into(self) -> [u8; 65] {
		self.0
	}
}

impl From<Signature> for H520 {
	fn from(s: Signature) -> Self {
		H520::from(s.0)
	}
}

impl From<H520> for Signature {
	fn from(bytes: H520) -> Self {
		Signature(bytes.into())
	}
}

impl Deref for Signature {
	type Target = [u8; 65];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Signature {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// Signs message with the given secret key.
/// Returns the corresponding signature.
pub fn sign(secret: &Secret, message: &Message) -> Result<Signature, Error> {
	let context = &SECP256K1;
	let sec = SecretKey::from_slice(secret.as_ref())?;
	let s = context.sign_recoverable(&SecpMessage::from_slice(&message[..])?, &sec);
	let (rec_id, data) = s.serialize_compact();
	let mut data_arr = [0; 65];

	// no need to check if s is low, it always is
	data_arr[0..64].copy_from_slice(&data[0..64]);
	data_arr[64] = rec_id.to_i32() as u8;
	Ok(Signature(data_arr))
}

/// Performs verification of the signature for the given message with corresponding public key
pub fn verify_public(public: &Public, signature: &Signature, message: &Message) -> Result<bool, Error> {
	let context = &SECP256K1;
	let rsig = RecoverableSignature::from_compact(&signature[0..64], RecoveryId::from_i32(signature[64] as i32)?)?;
	let sig = rsig.to_standard();

	let pdata: [u8; 65] = {
		let mut temp = [4u8; 65];
		temp[1..65].copy_from_slice(public.as_bytes());
		temp
	};

	let publ = PublicKey::from_slice(&pdata)?;
	match context.verify(&SecpMessage::from_slice(&message[..])?, &sig, &publ) {
		Ok(_) => Ok(true),
		Err(SecpError::IncorrectSignature) => Ok(false),
		Err(x) => Err(Error::from(x)),
	}
}

/// Checks if the address corresponds to the public key from the signature for the message
pub fn verify_address(address: &Address, signature: &Signature, message: &Message) -> Result<bool, Error> {
	let public = recover(signature, message)?;
	let recovered_address = public_to_address(&public);
	Ok(address == &recovered_address)
}

/// Recovers the public key from the signature for the message
pub fn recover(signature: &Signature, message: &Message) -> Result<Public, Error> {
	let rsig = RecoverableSignature::from_compact(&signature[0..64], RecoveryId::from_i32(signature[64] as i32)?)?;
	let pubkey = &SECP256K1.recover(&SecpMessage::from_slice(&message[..])?, &rsig)?;
	let serialized = pubkey.serialize_uncompressed();
	let mut public = Public::default();
	public.as_bytes_mut().copy_from_slice(&serialized[1..65]);
	Ok(public)
}

#[cfg(test)]
mod tests {
	use super::{
		super::{Generator, Message, Random},
		recover, sign, verify_address, verify_public, Signature,
	};
	use std::str::FromStr;

	#[test]
	fn vrs_conversion() {
		// given
		let keypair = Random.generate();
		let message = Message::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
		let signature = sign(keypair.secret(), &message).expect("can sign a non-zero message");

		// when
		let vrs = signature.clone().into_electrum();
		let from_vrs = Signature::from_electrum(&vrs);

		// then
		assert_eq!(signature, from_vrs);
	}

	#[test]
	fn signature_to_and_from_str() {
		let keypair = Random.generate();
		let message = Message::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
		let signature = sign(keypair.secret(), &message).expect("can sign a non-zero message");
		let string = format!("{}", signature);
		let deserialized = Signature::from_str(&string).unwrap();
		assert_eq!(signature, deserialized);
	}

	#[test]
	fn sign_and_recover_public() {
		let keypair = Random.generate();
		let message = Message::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
		let signature = sign(keypair.secret(), &message).unwrap();
		assert_eq!(keypair.public(), &recover(&signature, &message).unwrap());
	}

	#[test]
	fn sign_and_recover_public_works_with_zeroed_messages() {
		let keypair = Random.generate();
		let signature = sign(keypair.secret(), &Message::zero()).unwrap();
		let zero_message = Message::zero();
		assert_eq!(keypair.public(), &recover(&signature, &zero_message).unwrap());
	}

	#[test]
	fn recover_allowing_all_zero_message_can_recover_from_all_zero_messages() {
		let keypair = Random.generate();
		let signature = sign(keypair.secret(), &Message::zero()).unwrap();
		let zero_message = Message::zero();
		assert_eq!(keypair.public(), &recover(&signature, &zero_message).unwrap())
	}

	#[test]
	fn sign_and_verify_public() {
		let keypair = Random.generate();
		let message = Message::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
		let signature = sign(keypair.secret(), &message).expect("can sign a non-zero message");
		assert!(verify_public(keypair.public(), &signature, &message).unwrap());
	}

	#[test]
	fn sign_and_verify_address() {
		let keypair = Random.generate();
		let message = Message::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
		let signature = sign(keypair.secret(), &message).expect("can sign a non-zero message");
		assert!(verify_address(&keypair.address(), &signature, &message).unwrap());
	}
}
