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

use self::secp256k1::curve::{
	Affine,
	Jacobian,
	Scalar,
};

use self::secp256k1::curve::ECMULT_CONTEXT;


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
		// TODO find a way to clear secret, next lines break on mem replace
		//let key = std::mem::replace(&mut self.0, ZERO_KEY.0.clone());
		//let buf = &mut Into::<Scalar>::into(*key.inner).0;
		//Clear::clear(buf);
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
		let mut buf = [0;32];
		if message.len() != 32 {
			return Err(InnerError::InvalidMessage.into());
		}
		buf.copy_from_slice(&message[..]);
		let message = Message::parse(&buf);
		let mut buf = [0;64];
		if signature.len() < 65 {
			return Err(InnerError::InvalidSignature.into());
		}
		buf.copy_from_slice(&signature[..64]);
		let recovery_id = RecoveryId::parse(signature[64])?; 
		let signature = Signature::parse(&buf);
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
		let mut buf = [0;32];
		if message.len() != 32 {
			return Err(InnerError::InvalidMessage.into());
		}
		buf.copy_from_slice(&message[..]);
		let message = Message::parse(&buf);
		let mut buf = [0;64];
		if signature.len() < 64 {
			return Err(InnerError::InvalidSignature.into());
		}
		buf.copy_from_slice(&signature[..64]);
		let signature = Signature::parse(&buf);

		Ok(secp256k1::verify(&message, &signature, &self.0))
	}

}

impl SecretKeyTrait for SecretKey {
	type VecRepr = ClearOnDrop<Vec<u8>>;

	fn to_vec(&self) -> Self::VecRepr {
		ClearOnDrop::new(self.0.serialize().to_vec())
	}

	fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Error> {

	 	let mut buf = [0;32];
		if message.len() != 32 {
			return Err(InnerError::InvalidMessage.into());
		}
		buf.copy_from_slice(&message[..]);
		let message = Message::parse(&buf);
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

fn aff_to_public(aff_pub: &mut Affine) -> Result<PublicKeyInner, Error> {
	let mut buff = [4;65];
	let mut buff2 = [0;32];
	aff_pub.x.normalize();
	aff_pub.x.fill_b32(&mut buff2);
	buff[1..33].copy_from_slice(&buff2[..]);
	aff_pub.y.normalize();
	aff_pub.y.fill_b32(&mut buff2);
	buff[33..65].copy_from_slice(&buff2[..]);
	Ok(PublicKeyInner::parse(&buff)?)
}

struct SecretScalar(pub Scalar);

impl Drop for SecretScalar {
	fn drop(&mut self) {
		self.0.clear();
	}
}

impl FiniteField for Secp256k1 {

	fn generator_x() -> &'static[u8] { &GENERATOR_X[..] }
	fn generator_y() -> &'static[u8] { &GENERATOR_Y[..] }
	fn curve_order() -> &'static[u8] { &CURVE_ORDER[..] }

	fn public_mul(pub_key: &mut Self::PublicKey, sec_key: &Self::SecretKey) -> Result<(), Error> {
		let sec_scal = SecretScalar(sec_key.0.clone().into());
		let mut pub_aff: Affine = pub_key.0.clone().into();
		let mut pub_jac = Jacobian::default();
		pub_jac.set_ge(&pub_aff);
		//ECMULT_GEN_CONTEXT.ecmult_gen(&mut pub_jac, &sec_scal);
		//pub_aff.set_gej(&pub_jac);
		let mut zero = Scalar::default();
		zero.set_int(0);
		let mut res = Jacobian::default();
		ECMULT_CONTEXT.ecmult(&mut res, &pub_jac, &sec_scal.0, &zero);
		pub_aff.set_gej(&res);
		*pub_key = PublicKey::new(aff_to_public(&mut pub_aff)?);
		Ok(())
	}

	fn public_add(pub_key: &mut Self::PublicKey, other_public: &Self::PublicKey) -> Result<(), Error> {
		let mut aff_pub: Affine = pub_key.0.clone().into();
		let mut aff_pub_j = Jacobian::default();
		aff_pub_j.set_ge(&aff_pub);
		let aff_pub_other: Affine = other_public.0.clone().into();
		let res_j = aff_pub_j.add_ge(&aff_pub_other);
		aff_pub.set_gej(&res_j);
		*pub_key = PublicKey::new(aff_to_public(&mut aff_pub)?);

		Ok(())
	}

	fn secret_mul(sec_key: &mut Self::SecretKey, other_sec_key: &Self::SecretKey) -> Result<(), Error> {
		let sec_scal = SecretScalar(sec_key.0.clone().into());
		let other_sec_scal = SecretScalar(other_sec_key.0.clone().into());
		// we could use * operator instead.
		let mut res = SecretScalar(Scalar::default());
		res.0.mul_in_place(&sec_scal.0, &other_sec_scal.0);
		*sec_key = SecretKey::new(SecretKeyInner::parse(&res.0.b32())?);
		Ok(())
	}

	fn secret_add(sec_key: &mut Self::SecretKey, other_sec_key: &Self::SecretKey) -> Result<(), Error> {
		let sec_scal = SecretScalar(sec_key.0.clone().into());
		let other_sec_scal = SecretScalar(other_sec_key.0.clone().into());
		// we could use + operator instead.
		let mut res = SecretScalar(Scalar::default());
		res.0.add_in_place(&sec_scal.0, &other_sec_scal.0);
		*sec_key = SecretKey::new(SecretKeyInner::parse(&res.0.b32())?);
		Ok(())
	}

	fn secret_inv(sec_key: &mut Self::SecretKey) -> Result<(), Error> {
		let sec_scal = SecretScalar(sec_key.0.clone().into());
		let mut res = SecretScalar(Scalar::default());
		res.0.inv_in_place(&sec_scal.0);
		*sec_key = SecretKey::new(SecretKeyInner::parse(&res.0.b32())?);
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
		}
	}
}

#[cfg(test)]
type AsymTest = Secp256k1;

#[cfg(test)]
::tests_asym!();

