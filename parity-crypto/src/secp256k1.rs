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
//! TODO sized u8 array in proto should be usable if we add methods such as U256 -> &[u8;32] to ethereum_types
//! TODO use SecretKey and PublicKey explicitly in if (with conversion from &[u8]) : methods are
//! highly inefficient here.

extern crate secp256k1;
extern crate arrayvec;
extern crate rand;
use clear_on_drop::clear::Clear;

use self::arrayvec::ArrayVec;
use self::rand::Rng;
use super::traits::asym::{SecretKey as SecretKeyTrait, PublicKey as PublicKeyTrait, Asym, FiniteField, FixAsymSharedSecret};

use super::error::Error;

// reexports
pub use self::secp256k1::{
	Error as InnerError,
};

pub use self::secp256k1::key::{SecretKey as SecretKeyInner, PublicKey as PublicKeyInner};
use self::secp256k1::constants::{SECRET_KEY_SIZE, GENERATOR_X, GENERATOR_Y, CURVE_ORDER};

use self::secp256k1::key::{ZERO_KEY as ZERO_BYTES, ONE_KEY as ONE_BYTES, MINUS_ONE_KEY as MINUS_ONE_BYTES};
use self::secp256k1::{
	Message,
	RecoverableSignature,
	RecoveryId,
	ecdh,
};

lazy_static! {
	pub static ref SECP256K1: self::secp256k1::Secp256k1 = self::secp256k1::Secp256k1::new();
	static ref MINUS_ONE_KEY: SecretKey = SecretKey(MINUS_ONE_BYTES);
	static ref ONE_KEY: SecretKey = SecretKey(ONE_BYTES);
	static ref ZERO_KEY: SecretKey = SecretKey(ZERO_BYTES);
	static ref NULL_PUB_K: PublicKey = PublicKey::unsafe_empty();
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SharedSecretAsRef(pub ecdh::SharedSecret);

impl AsRef<[u8]> for SharedSecretAsRef {
	fn as_ref(&self) -> &[u8] {
		&self.0[..]
	}
}

const SIGN_SIZE: usize = 65;
const PUB_SIZE: usize = 64;

// not vec size could be reduce to 64 (higher instantiation cost)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PublicKey(PublicKeyInner, ArrayVec<[u8;72]>);

impl PublicKey {

	fn new(inner: PublicKeyInner) -> Self {
		let a_vec = inner.serialize_vec(&SECP256K1, false);
		PublicKey(inner, a_vec)
	}

	fn refresh(&mut self) {
		let a_vec = self.0.serialize_vec(&SECP256K1, false);
		self.1 = a_vec;
	}

	fn unsafe_empty() -> Self {
		PublicKey(PublicKeyInner::new(), [0;72].into())
	}
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SecretKey(pub SecretKeyInner);

impl Drop for SecretKey {
	fn drop(&mut self) {
		let len = self.0.len();
		unsafe {
			let mut v = std::slice::from_raw_parts(self.0.as_mut_ptr(), len);
			v.clear()
		}
	}
}

impl Asym for Secp256k1 {
	type PublicKey = PublicKey;
	type SecretKey = SecretKey;

	const SIGN_SIZE: usize = SIGN_SIZE;

	/// Warning we use 64 bit pubsize (first bytes of 65 bit representation is 4).
	const PUB_SIZE: usize = PUB_SIZE;

	const SECRET_SIZE: usize = SECRET_KEY_SIZE;

	fn recover(signature: &[u8], message: &[u8]) -> Result<Self::PublicKey, Error> {
		let context = &SECP256K1;
		let rsig = RecoverableSignature::from_compact(context, &signature[0..PUB_SIZE], RecoveryId::from_i32(signature[PUB_SIZE] as i32)?)?;
		let pubkey = context.recover(&Message::from_slice(message)?, &rsig)?;
		Ok(PublicKey::new(pubkey))
		//let serialized = pubkey.serialize_vec(context, false);

		//let mut res = vec![0;PUB_SIZE];
		//res[..].copy_from_slice(&serialized[1..PUB_SIZE + 1]);
		//Ok(res)
	}


	/// deprecated, we rather not expose Rng trait, use `keypair_from_slice` instead.
	/// The intent is to avoid depending on `Rng` trait.
	fn generate_keypair(r: &mut impl Rng) -> (Self::SecretKey, Self::PublicKey) {
		let (s, p) = SECP256K1.generate_keypair(r)
			.expect("context always created with full capabilities; qed");
		(SecretKey(s), PublicKey::new(p))
	}

	/// create a key pair from byte value of the secret key, the calling function is responsible for
	/// erasing the input of memory.
	fn keypair_from_slice(sk_bytes: &[u8]) -> Result<(Self::SecretKey, Self::PublicKey), Error> {
		assert!(sk_bytes.len() == SECRET_KEY_SIZE);
		let sk = SecretKeyInner::from_slice(&SECP256K1, sk_bytes)?;
		let sc = SecretKey(sk);
		let pk = PublicKeyInner::from_secret_key(&SECP256K1, &sc.0)?;
		Ok((sc, PublicKey::new(pk)))
	}

	fn public_from_secret(s: &Self::SecretKey) -> Result<Self::PublicKey, Error> {
		Ok(PublicKey::new(PublicKeyInner::from_secret_key(&SECP256K1, &s.0)?))
	}

	/// using a shortened 64bit public key as input
	fn public_from_slice(public_sec_raw: &[u8]) -> Result<Self::PublicKey, Error> {
		let pdata = {
			let mut temp = [4u8; PUB_SIZE + 1];
			(&mut temp[1..PUB_SIZE + 1]).copy_from_slice(&public_sec_raw[0..PUB_SIZE]);
			temp
		};
		Ok(PublicKey::new(PublicKeyInner::from_slice(&SECP256K1, &pdata)?))
	}

	fn secret_from_slice(secret: &[u8]) -> Result<Self::SecretKey, Error> {
		Ok(SecretKey(SecretKeyInner::from_slice(&SECP256K1, secret)?))
	}



}

impl FixAsymSharedSecret for SecretKey {
	type Other = PublicKey;
	type Result = SharedSecretAsRef;

	fn shared_secret(&self, publ: &Self::Other) -> Result<Self::Result, Error> {
		let shared = ecdh::SharedSecret::new_raw(&SECP256K1, &publ.0, &self.0);
		Ok(SharedSecretAsRef(shared))
	}

}

impl SecretKeyTrait for SecretKey {
	//type VecRepr = ClearOnDrop<Vec<u8>>;

	fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Error> {
		let context = &SECP256K1;
		let s = context.sign_recoverable(&Message::from_slice(message)?, &self.0)?;
		let (rec_id, data) = s.serialize_compact(context);
		let mut data_arr = vec![0; SIGN_SIZE];

		// no need to check if s is low, it always is
		data_arr[0..PUB_SIZE].copy_from_slice(&data[0..PUB_SIZE]);
		data_arr[PUB_SIZE] = rec_id.to_i32() as u8;
		Ok(data_arr)
	}

	/*fn to_vec(&self) -> Self::VecRepr {
		ClearOnDrop::new(self.0[..].to_vec())
	}*/

}

impl AsRef<[u8]> for SecretKey {
	fn as_ref(&self) -> &[u8] {
		&self.0[..]
	}
}

impl PublicKeyTrait for PublicKey {
	type VecRepr = ArrayVec<[u8; 72]>;

	/*fn to_vec(&self) -> Self::VecRepr {
		let mut a_vec = self.serialize_vec(&SECP256K1, false);
		let _ = a_vec.drain(PUB_SIZE + 1..);
		a_vec.remove(0);
		a_vec
	}*/

	/// Should move to another trait.
	fn to_compressed_vec(p: &Self) -> Self::VecRepr {
		p.0.serialize_vec(&SECP256K1, true)
	}

	fn is_valid(&self) -> bool {
		self.0.is_valid()
	}

	fn verify(&self, signature: &[u8], message: &[u8]) -> Result<bool, Error> {
		let context = &SECP256K1;
		let rsig = RecoverableSignature::from_compact(context, &signature[0..PUB_SIZE], RecoveryId::from_i32(signature[PUB_SIZE] as i32)?)?;
		let sig = rsig.to_standard(context);

		match context.verify(&Message::from_slice(message)?, &sig, &self.0) {
			Ok(_) => Ok(true),
			Err(InnerError::IncorrectSignature) => Ok(false),
			Err(x) => Err(InnerError::from(x).into())
		}
	}

}

// warning it returns PUB_SIZE byte vec (we skip the first byte of SIGN_SIZE byte more standard
// representation)
impl AsRef<[u8]> for PublicKey {
	fn as_ref(&self) -> &[u8] {
		&self.1[1 .. 1 + PUB_SIZE]
	}
}


pub struct Secp256k1;

impl FiniteField for Secp256k1 {

	fn generator_x() -> &'static[u8] { &GENERATOR_X[..] }
	fn generator_y() -> &'static[u8] { &GENERATOR_Y[..] }
	fn curve_order() -> &'static[u8] { &CURVE_ORDER[..] }

	fn public_mul(pub_key: &mut Self::PublicKey, sec_key: &Self::SecretKey) -> Result<(), Error> {
		pub_key.0.mul_assign(&SECP256K1, &sec_key.0)?;
		pub_key.refresh();
		Ok(())
	}

	fn public_add(pub_key: &mut Self::PublicKey, other_public: &Self::PublicKey) -> Result<(), Error> {
		pub_key.0.add_assign(&SECP256K1, &other_public.0)?;
		pub_key.refresh();
		Ok(())
	}

	fn secret_mul(sec_key: &mut Self::SecretKey, other_secret: &Self::SecretKey) -> Result<(), Error> {
		sec_key.0.mul_assign(&SECP256K1, &other_secret.0)?;
		Ok(())
	}

	fn secret_add(sec_key: &mut Self::SecretKey, other_secret: &Self::SecretKey) -> Result<(), Error> {
		sec_key.0.add_assign(&SECP256K1, &other_secret.0)?;
		Ok(())
	}

	fn secret_inv(sec_key: &mut Self::SecretKey) -> Result<(), Error> {
		sec_key.0.inv_assign(&SECP256K1)?;
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
			InnerError::InvalidSignature |
			InnerError::IncorrectSignature => Error::AsymShort("Invalid EC signature"),
			InnerError::InvalidMessage => Error::AsymShort("Invalid AES message"),
			_ => Error::AsymFull(Box::new(err))
		}
	}
}

#[cfg(test)]
mod tests {
	extern crate rand;
	use ::traits::asym::*;
	use super::{
		Secp256k1,
	};
	use self::rand::OsRng;
	use self::rand::Rng;

	#[test]
	fn sign_val() {
		let sk = [213, 68, 220, 102, 106, 158, 142, 136, 198, 84, 32, 178, 49, 72, 194, 143, 116, 165, 155, 122, 20, 120, 169, 29, 129, 128, 206, 190, 48, 122, 97, 52];
		let sec = Secp256k1::secret_from_slice(&sk).unwrap();
		let message = vec![2;32];
		let signature = sec.sign(&message).unwrap();
		assert_eq!(&signature[..], &[88, 96, 150, 252, 139, 37, 138, 196, 9, 30, 22, 98, 125, 20, 223, 16, 221, 46, 42, 225, 164, 71, 221, 37, 81, 9, 58, 3, 31, 245, 121, 110, 0, 248, 154, 65, 12, 193, 151, 151, 236, 69, 230, 56, 39, 161, 124, 1, 30, 20, 130, 5, 174, 75, 254, 199, 5, 119, 39, 223, 20, 116, 11, 229, 0][..]);
	}

	#[test]
	fn sign_and_recover_public() {
		let mut osrng = OsRng::new().expect("test");
		let mut sec_buf = vec![0; Secp256k1::SECRET_SIZE];
		osrng.fill_bytes(&mut sec_buf[..]);
		let (secret, public) = Secp256k1::keypair_from_slice(&mut sec_buf).unwrap();
		let message = vec![2;32];
		let signature = secret.sign(&message).unwrap();
		assert_eq!(public, Secp256k1::recover(&signature, &message).unwrap());
	}

	#[test]
	fn sign_and_verify_public() {
		let mut osrng = OsRng::new().expect("test");
		let mut sec_buf = vec![0; Secp256k1::SECRET_SIZE];
		osrng.fill_bytes(&mut sec_buf[..]);
		let (secret, public) = Secp256k1::keypair_from_slice(&mut sec_buf).unwrap();
		let message = vec![0;32];
		let signature = secret.sign(&message).unwrap();
		assert!(public.verify(&signature, &message).unwrap());
	}

	#[test]
	fn public_addition() {
		let pk1 = [126, 60, 36, 91, 73, 177, 194, 111, 11, 3, 99, 246, 204, 86, 122, 109, 85, 28, 43, 169, 243, 35, 76, 152, 90, 76, 241, 17, 108, 232, 215, 115, 15, 19, 23, 164, 151, 43, 28, 44, 59, 141, 167, 134, 112, 105, 251, 15, 193, 183, 224, 238, 154, 204, 230, 163, 216, 235, 112, 77, 239, 98, 135, 132];
		let pk2 = [40, 127, 167, 223, 38, 53, 6, 223, 67, 83, 204, 60, 226, 227, 107, 231, 172, 34, 3, 187, 79, 112, 167, 0, 217, 118, 69, 218, 189, 208, 150, 190, 54, 186, 220, 95, 80, 220, 183, 202, 117, 160, 18, 84, 245, 181, 23, 32, 51, 73, 178, 173, 92, 118, 92, 122, 83, 49, 54, 195, 194, 16, 229, 39];
		let mut pub1 = Secp256k1::public_from_slice(&pk1[..]).unwrap();
		let pub2 = Secp256k1::public_from_slice(&pk2[..]).unwrap();
		Secp256k1::public_add(&mut pub1, &pub2).unwrap();

		assert_eq!(&pub1.as_ref()[..], &[101, 166, 20, 152, 34, 76, 121, 113, 139, 80, 13, 92, 122, 96, 38, 194, 205, 149, 93, 19, 147, 132, 195, 173, 42, 86, 26, 221, 170, 127, 180, 168, 145, 21, 75, 45, 248, 90, 114, 118, 62, 196, 194, 143, 245, 204, 184, 16, 175, 202, 175, 228, 207, 112, 219, 94, 237, 75, 105, 186, 56, 102, 46, 147][..]);
	}

	#[test]
	fn public_multiplication() {
		let pk = [126, 60, 36, 91, 73, 177, 194, 111, 11, 3, 99, 246, 204, 86, 122, 109, 85, 28, 43, 169, 243, 35, 76, 152, 90, 76, 241, 17, 108, 232, 215, 115, 15, 19, 23, 164, 151, 43, 28, 44, 59, 141, 167, 134, 112, 105, 251, 15, 193, 183, 224, 238, 154, 204, 230, 163, 216, 235, 112, 77, 239, 98, 135, 132];
		let sk = [213, 68, 220, 102, 106, 158, 142, 136, 198, 84, 32, 178, 49, 72, 194, 143, 116, 165, 155, 122, 20, 120, 169, 29, 129, 128, 206, 190, 48, 122, 97, 52];
		let mut pubk = Secp256k1::public_from_slice(&pk[..]).unwrap();
		let sec = Secp256k1::secret_from_slice(&sk[..]).unwrap();
		Secp256k1::public_mul(&mut pubk, &sec).unwrap();

		assert_eq!(&pubk.as_ref()[..], &[98, 132, 11, 170, 93, 231, 41, 185, 180, 151, 185, 130, 77, 251, 41, 169, 160, 84, 133, 19, 82, 190, 137, 82, 0, 214, 148, 120, 165, 184, 17, 21, 237, 184, 119, 174, 13, 77, 50, 251, 16, 17, 197, 74, 232, 55, 142, 220, 27, 152, 4, 52, 69, 14, 76, 8, 156, 82, 0, 193, 179, 65, 63, 106][..]);
	}


	#[test]
	fn public_addition_is_commutative() {
		let mut osrng = OsRng::new().expect("test");
		let mut sec_buf = vec![0; Secp256k1::SECRET_SIZE];
		osrng.fill_bytes(&mut sec_buf[..]);
		let (_, public1) = Secp256k1::keypair_from_slice(&mut sec_buf).unwrap();
		osrng.fill_bytes(&mut sec_buf[..]);
		let (_, public2) = Secp256k1::keypair_from_slice(&mut sec_buf).unwrap();

		let mut left = public1.clone();
		Secp256k1::public_add(&mut left, &public2).unwrap();

		let mut right = public2.clone();
		Secp256k1::public_add(&mut right, &public1).unwrap();

		assert_eq!(left, right);
	}

	#[test]
	fn public_addition_is_reversible_with_subtraction() {
		let mut osrng = OsRng::new().expect("test");
		let mut sec_buf = vec![0; Secp256k1::SECRET_SIZE];
		osrng.fill_bytes(&mut sec_buf[..]);
		let (_, public1) = Secp256k1::keypair_from_slice(&mut sec_buf).unwrap();
		osrng.fill_bytes(&mut sec_buf[..]);
		let (_, public2) = Secp256k1::keypair_from_slice(&mut sec_buf).unwrap();

		let mut sum = public1.clone();
		Secp256k1::public_add(&mut sum, &public2).unwrap();
		let mut op = public2.clone();
		Secp256k1::public_mul(&mut op, Secp256k1::minus_one_key()).unwrap();
		Secp256k1::public_add(&mut sum, &op).unwrap();

		assert_eq!(sum, public1);
	}


	#[test]
	fn multiplicating_secret_inversion_with_secret_gives_one() {
		let mut osrng = OsRng::new().expect("test");
		let mut sec_buf = vec![0; Secp256k1::SECRET_SIZE];
		osrng.fill_bytes(&mut sec_buf[..]);
		let (secret, _) = Secp256k1::keypair_from_slice(&mut sec_buf).unwrap();

		let mut inversion = secret.clone();
		Secp256k1::secret_inv(&mut inversion).unwrap();
		Secp256k1::secret_mul(&mut inversion, &secret).unwrap();
		assert_eq!(inversion, *Secp256k1::one_key());
	}

	#[test]
	fn secret_inversion_is_reversible_with_inversion() {
		let mut osrng = OsRng::new().expect("test");
		let mut sec_buf = vec![0; Secp256k1::SECRET_SIZE];
		osrng.fill_bytes(&mut sec_buf[..]);
		let (secret, _) = Secp256k1::keypair_from_slice(&mut sec_buf).unwrap();
		let mut inversion = secret.clone();
		Secp256k1::secret_inv(&mut inversion).unwrap();
		Secp256k1::secret_inv(&mut inversion).unwrap();
		assert_eq!(inversion, secret);
	}

}

/// Default implementation is only for parity-ethereum secret-store
/// It would be good to remove it (there is a bit of refactoring).
/// Therefore the constraint is not explicit.
/// Please note that it is an invalid publickey.
impl Default for PublicKey {
	fn default() -> Self {
		NULL_PUB_K.clone()
	}
}
