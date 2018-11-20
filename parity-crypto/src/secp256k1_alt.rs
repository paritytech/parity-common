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

//! secp256k1 for parity.

extern crate libsecp256k1 as secp256k1;

use clear_on_drop::clear::Clear;
use clear_on_drop::ClearOnDrop;
use ::traits::asym::{
	Asym,
	PublicKey as PublicKeyTrait,
	SecretKey as SecretKeyTrait,
	FixAsymSharedSecret,
	FiniteField
};

use super::error::Error;

pub struct Secp256k1;

pub use self::secp256k1::{
	Error as InnerError,
	PublicKey as PublicKeyInner,
	SecretKey as SecretKeyInner,
};


use self::secp256k1::{
	Message,
	Signature,
	RecoveryId,
};

const SIGN_SIZE: usize = 65;
const PUB_SIZE: usize = 64;
const SECRET_SIZE: usize = 32;

const MINUS_ONE_BYTES: [u8;32] = [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 254, 186, 174, 220, 230, 175, 72, 160, 59, 191, 210, 94, 140, 208, 54, 65, 64];

const ONE_BYTES: [u8;32] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];

const ZERO_BYTES: [u8;32] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

/// The X coordinate of the generator (could get from lib AFFINE_G const but it is more convenient
/// this way : could be default value of a trait)
pub const GENERATOR_X: [u8; 32] = [
	0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac,
	0x55, 0xa0, 0x62, 0x95, 0xce, 0x87, 0x0b, 0x07,
	0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9,
	0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8, 0x17, 0x98
];

/// The Y coordinate of the generator
pub const GENERATOR_Y: [u8; 32] = [
	0x48, 0x3a, 0xda, 0x77, 0x26, 0xa3, 0xc4, 0x65,
	0x5d, 0xa4, 0xfb, 0xfc, 0x0e, 0x11, 0x08, 0xa8,
	0xfd, 0x17, 0xb4, 0x48, 0xa6, 0x85, 0x54, 0x19,
	0x9c, 0x47, 0xd0, 0x8f, 0xfb, 0x10, 0xd4, 0xb8
];

/// The order of the secp256k1 curve
pub const CURVE_ORDER: [u8; 32] = [
	0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
	0xba, 0xae, 0xdc, 0xe6, 0xaf, 0x48, 0xa0, 0x3b,
	0xbf, 0xd2, 0x5e, 0x8c, 0xd0, 0x36, 0x41, 0x41
];

lazy_static! {
	static ref MINUS_ONE_KEY: SecretKey = SecretKey::new(SecretKeyInner::parse(&MINUS_ONE_BYTES).expect("static; qed"));
	static ref ONE_KEY: SecretKey = SecretKey::new(SecretKeyInner::parse(&ONE_BYTES).expect("static; qed"));
	static ref ZERO_KEY: SecretKey = SecretKey::new(SecretKeyInner::parse(&ZERO_BYTES).expect("static; qed"));
}

pub fn one_key() -> &'static SecretKey {
	&ONE_KEY
}

pub fn minus_one_key() -> &'static SecretKey {
	&MINUS_ONE_KEY
}


#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PublicKey(PublicKeyInner);

impl PublicKey {

	fn new(inner: PublicKeyInner) -> Self {
		PublicKey(inner)
	}

}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SecretKey(SecretKeyInner);

impl Drop for SecretKey {
	fn drop(&mut self) {
		Clear::clear(&mut self.0);
	}
}

impl SecretKey {

	fn new(inner: SecretKeyInner) -> Self {
		SecretKey(inner)
	}

}

impl Asym for Secp256k1 {
	type PublicKey = PublicKey;
	type SecretKey = SecretKey;

	/// This is highly ethereum opignated
	const SIGN_SIZE: usize = SIGN_SIZE;

	/// Warning we use 64 bit pubsize (first bytes of 65 bit representation is 4).
	const PUB_SIZE: usize = PUB_SIZE;

	const SECRET_SIZE: usize = SECRET_SIZE;

	const KEYPAIR_INPUT_SIZE: usize = Self::SECRET_SIZE;

	fn recover(signature: &[u8], message: &[u8]) -> Result<Self::PublicKey, Error> {
		let message = Message::parse_slice(&message[..])?;
		if signature.len() < 65 {
			return Err(InnerError::InvalidSignature.into());
		}
		let recovery_id = RecoveryId::parse(signature[64])?; 
		let signature = Signature::parse_slice(&signature[..64])?;
		let public_key = secp256k1::recover(&message, &signature, &recovery_id)?;
		Ok(PublicKey::new(public_key))
	}

	/// create a key pair from byte value of the secret key, the calling function is responsible for
	/// erasing the input of memory.
	fn keypair_from_slice(sk_bytes: &[u8]) -> Result<(Self::SecretKey, Self::PublicKey), Error> {
		assert!(sk_bytes.len() == SECRET_SIZE);
		let secret_key = Self::secret_from_slice(sk_bytes)?;
		let public_key = Self::public_from_secret(&secret_key)?;
		Ok((secret_key, public_key))
	}

	fn public_from_secret(s: &Self::SecretKey) -> Result<Self::PublicKey, Error> {
		Ok(PublicKey::new(PublicKeyInner::from_secret_key(&s.0)))
	}

	/// using a shortened 64bit public key as input
	fn public_from_slice(public_sec_raw: &[u8]) -> Result<Self::PublicKey, Error> {
		if public_sec_raw.len() < PUB_SIZE {
			return Err(InnerError::InvalidPublicKey.into());
		}
		let pdata = {
			let mut temp = [4u8; PUB_SIZE + 1];
			(&mut temp[1..PUB_SIZE + 1]).copy_from_slice(&public_sec_raw[..PUB_SIZE]);
			temp
		};
		Ok(PublicKey::new(PublicKeyInner::parse(&pdata)?))
	}

	fn secret_from_slice(secret: &[u8]) -> Result<Self::SecretKey, Error> {
		if secret.len() < SECRET_SIZE {
			return Err(InnerError::InvalidSecretKey.into());
		}
		let mut buf = [0;32];
		buf[..].copy_from_slice(&secret[..SECRET_SIZE]);
		let res = SecretKey::new(SecretKeyInner::parse(&buf)?);
		Clear::clear(&mut buf);
		Ok(res)
	}


}

impl PublicKeyTrait for PublicKey {
	type VecRepr = Vec<u8>;
	type CompVecRepr = Vec<u8>;

	fn to_vec(&self) -> Self::VecRepr {
		self.0.serialize()[1..PUB_SIZE + 1].to_vec()
	}

	/// Should move to another trait.
	fn to_compressed_vec(&self) -> Self::CompVecRepr {
		self.0.serialize_compressed().to_vec()
	}

	fn verify(&self, signature: &[u8], message: &[u8]) -> Result<bool, Error> {
		let message = Message::parse_slice(&message[..])?;
		if signature.len() < 64 {
			return Err(InnerError::InvalidSignature.into());
		}
		let signature = Signature::parse_slice(&signature[..64])?;

		Ok(secp256k1::verify(&message, &signature, &self.0))
	}

}

impl SecretKeyTrait for SecretKey {
	type VecRepr = ClearOnDrop<Vec<u8>>;

	fn to_vec(&self) -> Self::VecRepr {
		ClearOnDrop::new(self.0.serialize().to_vec())
	}

	fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Error> {
		let message = Message::parse_slice(&message[..])?;
		let (sig, rec_id) = secp256k1::sign(&message, &self.0)?;
		let mut data_arr = vec![0; 65];
		data_arr[0..64].copy_from_slice(&sig.serialize());
		data_arr[64] = rec_id.serialize();
		Ok(data_arr)
	}

}

impl FixAsymSharedSecret for SecretKey {
	type Other = PublicKey;
	type Result = SharedSecretAsRef;

	fn shared_secret(&self, publ: &Self::Other) -> Result<Self::Result, Error> {
		let shared = secp256k1::SharedSecret::new(&publ.0, &self.0)?;
		Ok(SharedSecretAsRef(shared))
	}

}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SharedSecretAsRef(pub secp256k1::SharedSecret);

impl AsRef<[u8]> for SharedSecretAsRef {
	fn as_ref(&self) -> &[u8] {
		self.0.as_ref()
	}
}

impl FiniteField for Secp256k1 {

	fn generator_x() -> &'static[u8] { &GENERATOR_X[..] }
	fn generator_y() -> &'static[u8] { &GENERATOR_Y[..] }
	fn curve_order() -> &'static[u8] { &CURVE_ORDER[..] }

	fn public_mul(pub_key: &mut Self::PublicKey, sec_key: &Self::SecretKey) -> Result<(), Error> {
		pub_key.0.tweak_mul_assign(&sec_key.0)?;
		Ok(())
	}

	fn public_add(pub_key: &mut Self::PublicKey, other_public: &Self::PublicKey) -> Result<(), Error> {
		let keys = [other_public.0.clone(), pub_key.0.clone()];
		*pub_key = PublicKey::new(PublicKeyInner::combine(&keys)?);
		Ok(())
	}

	fn secret_mul(sec_key: &mut Self::SecretKey, other_sec_key: &Self::SecretKey) -> Result<(), Error> {
		sec_key.0.tweak_mul_assign(&other_sec_key.0)?;
		Ok(())
	}

	fn secret_add(sec_key: &mut Self::SecretKey, other_sec_key: &Self::SecretKey) -> Result<(), Error> {
		sec_key.0.tweak_add_assign(&other_sec_key.0)?;
		Ok(())
	}

	fn secret_inv(sec_key: &mut Self::SecretKey) -> Result<(), Error> {
		*sec_key = SecretKey::new(sec_key.0.inv());
		Ok(())
	}

	fn one_key() -> &'static Self::SecretKey {
		&ONE_KEY
	}

	fn zero_key() -> &'static Self::SecretKey {
		&ZERO_KEY
	}

	fn minus_one_key() -> &'static Self::SecretKey {
		&MINUS_ONE_KEY
	}

}

impl From<InnerError> for Error {
	fn from(err: InnerError) -> Self {
		match err {
			InnerError::InvalidSecretKey => Error::AsymShort("Invalid secret"),
			InnerError::InvalidRecoveryId => Error::AsymShort("Invalid recovery id"),
			InnerError::InvalidPublicKey => Error::AsymShort("Invalid public"),
			InnerError::InvalidSignature => Error::AsymShort("Invalid EC signature"),
			InnerError::InvalidMessage => Error::AsymShort("Invalid AES message"),
			InnerError::InvalidInputLength => Error::AsymShort("Invalid input Length"),
			InnerError::TweakOutOfRange => Error::AsymShort("Tweak out of Range"),
		}
	}
}

#[cfg(test)]
type AsymTest = Secp256k1;

#[cfg(test)]
::tests_asym!();

