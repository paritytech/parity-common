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
extern crate rand;

use self::rand::Rng;

// reexports
pub use self::secp256k1::{
	Error,
};

pub use self::secp256k1::key::{SecretKey, PublicKey};
pub use self::secp256k1::constants::{SECRET_KEY_SIZE, GENERATOR_X, GENERATOR_Y, CURVE_ORDER};

use self::secp256k1::key::{ONE_KEY, MINUS_ONE_KEY};
use self::secp256k1::{
	Message,
	RecoverableSignature,
	RecoveryId,
	ecdh,
};

lazy_static! {
	pub static ref SECP256K1: self::secp256k1::Secp256k1 = self::secp256k1::Secp256k1::new();
}

pub fn one_key() -> &'static SecretKey {
	&ONE_KEY
}

pub fn minus_one_key() -> &'static SecretKey {
	&MINUS_ONE_KEY
}



pub fn sign(secret: &[u8], message: &[u8]) -> Result<[u8;65], Error> {
	let context = &SECP256K1;
	let sec = SecretKey::from_slice(context, &secret[..])?;
	let s = context.sign_recoverable(&Message::from_slice(message)?, &sec)?;
	let (rec_id, data) = s.serialize_compact(context);
	let mut data_arr = [0; 65];

	// no need to check if s is low, it always is
	data_arr[0..64].copy_from_slice(&data[0..64]);
	data_arr[64] = rec_id.to_i32() as u8;
	Ok(data_arr)
}

/// TODO use public as 65 instead ?? (here it is 64 but serialize as 65 usually) -> at least put a
/// big doc about that!!
pub fn verify_public(public: &[u8], signature: &[u8], message: &[u8]) -> Result<bool, Error> {
	let context = &SECP256K1;
	let rsig = RecoverableSignature::from_compact(context, &signature[0..64], RecoveryId::from_i32(signature[64] as i32)?)?;
	let sig = rsig.to_standard(context);

	let pdata: [u8; 65] = {
		let mut temp = [4u8; 65];
		temp[1..65].copy_from_slice(&*public);
		temp
	};

	let publ = PublicKey::from_slice(context, &pdata)?;
	match context.verify(&Message::from_slice(message)?, &sig, &publ) {
		Ok(_) => Ok(true),
		Err(Error::IncorrectSignature) => Ok(false),
		Err(x) => Err(Error::from(x))
	}
}

pub fn recover(signature: &[u8], message: &[u8]) -> Result<[u8;64], Error> {
	let context = &SECP256K1;
	let rsig = RecoverableSignature::from_compact(context, &signature[0..64], RecoveryId::from_i32(signature[64] as i32)?)?;
	let pubkey = context.recover(&Message::from_slice(message)?, &rsig)?;
	let serialized = pubkey.serialize_vec(context, false);

	let mut res = [0;64];
	res.copy_from_slice(&serialized[1..65]);
	Ok(res)
}


pub fn generate_keypair(r: &mut impl Rng) -> (SecretKey, PublicKey) {
		SECP256K1.generate_keypair(r)
			.expect("context always created with full capabilities; qed")
}

// TODO change it to slicable u8 return type
// Plus add comment explaining first bit removal
/// warning this returns 64 byte vec (we skip the first byte of 65 byte more standard
/// representation)
pub fn public_to_vec(p: &PublicKey) -> impl AsRef<[u8]> {
	let mut a_vec = p.serialize_vec(&SECP256K1, false);
	// &a_vec[1..65]
	let _ = a_vec.drain(65..);
	a_vec.remove(0);
	a_vec
}

pub fn public_is_valid(p: &PublicKey) -> bool {
	p.is_valid()
}

/// only for test (or make the result erasable)
pub fn secret_to_vec(p: &SecretKey) -> impl AsRef<[u8]> {
	p[..].to_vec()
}

pub fn public_to_compressed_vec(p: &PublicKey) -> impl AsRef<[u8]> {
	p.serialize_vec(&SECP256K1, true)
}

pub fn secret_from_slice(secret: &[u8]) -> Result<SecretKey, Error> {
	SecretKey::from_slice(&SECP256K1, secret)
}

pub struct SharedSecretAsRef(pub ecdh::SharedSecret);

impl AsRef<[u8]> for SharedSecretAsRef {
	fn as_ref(&self) -> &[u8] {
		&self.0[..]
	}
}

pub fn shared_secret(publ: &PublicKey, sec: &SecretKey) -> Result<impl AsRef<[u8]>, Error> {
	let shared = ecdh::SharedSecret::new_raw(&SECP256K1, &publ, &sec);
	Ok(SharedSecretAsRef(shared))
}
	
/// using a shortened 64bit public key as input
pub fn public_from_slice(public_sec_raw: &[u8]) -> Result<PublicKey, Error> {
	let pdata = {
		let mut temp = [4u8; 65];
		(&mut temp[1..65]).copy_from_slice(&public_sec_raw[0..64]);
		temp
	};

	PublicKey::from_slice(&SECP256K1, &pdata)
}

pub fn public_from_secret(s: &SecretKey) -> Result<PublicKey, Error> {
	PublicKey::from_secret_key(&SECP256K1, &s)
}

pub fn public_mul(mut pub_key: PublicKey, sec_key: &SecretKey) -> Result<PublicKey, Error> {
	pub_key.mul_assign(&SECP256K1, sec_key)?;
	Ok(pub_key)
}

pub fn public_add(mut pub_key: PublicKey, other_public: &PublicKey) -> Result<PublicKey, Error> {
	pub_key.add_assign(&SECP256K1, other_public)?;
	Ok(pub_key)
}

pub fn secret_mul(mut sec_key: SecretKey, other_secret: &SecretKey) -> Result<SecretKey, Error> {
	sec_key.mul_assign(&SECP256K1, other_secret)?;
	Ok(sec_key)
}

pub fn secret_add(mut sec_key: SecretKey, other_secret: &SecretKey) -> Result<SecretKey, Error> {
	sec_key.add_assign(&SECP256K1, other_secret)?;
	Ok(sec_key)
}

pub fn secret_inv(mut sec_key: SecretKey) -> Result<SecretKey, Error> {
	sec_key.inv_assign(&SECP256K1)?;
	Ok(sec_key)
}




#[cfg(test)]
mod tests {
	extern crate rand;
	use super::{
		sign,
		secret_from_slice,
		verify_public,
		recover,
		generate_keypair,
		public_to_vec,
		secret_to_vec,
		public_add,
		public_from_slice,
		public_mul,
		minus_one_key,
		secret_mul,
		secret_inv,
		one_key,
	};
	use self::rand::OsRng;

	#[test]
	fn sign_val() {
		let sk = [213, 68, 220, 102, 106, 158, 142, 136, 198, 84, 32, 178, 49, 72, 194, 143, 116, 165, 155, 122, 20, 120, 169, 29, 129, 128, 206, 190, 48, 122, 97, 52];
		let message = vec![2;32];
		let signature = sign(&sk[..], &message).unwrap();
		assert_eq!(&signature[..], &[88, 96, 150, 252, 139, 37, 138, 196, 9, 30, 22, 98, 125, 20, 223, 16, 221, 46, 42, 225, 164, 71, 221, 37, 81, 9, 58, 3, 31, 245, 121, 110, 0, 248, 154, 65, 12, 193, 151, 151, 236, 69, 230, 56, 39, 161, 124, 1, 30, 20, 130, 5, 174, 75, 254, 199, 5, 119, 39, 223, 20, 116, 11, 229, 0][..]);
	}

	#[test]
	fn sign_and_recover_public() {
		let mut osrng = OsRng::new().expect("test");
		let (secret, public) = generate_keypair(&mut osrng);
		let message = vec![2;32];
		let signature = sign(secret_to_vec(&secret).as_ref(), &message).unwrap();
		assert_eq!(&public_to_vec(&public).as_ref()[..], &recover(&signature, &message).unwrap()[..]);
	}

	#[test]
	fn sign_and_verify_public() {
		let mut osrng = OsRng::new().expect("test");
		let (secret, public) = generate_keypair(&mut osrng);
		let message = vec![0;32];
		let signature = sign(secret_to_vec(&secret).as_ref(), &message).unwrap();
		assert!(verify_public(&public_to_vec(&public).as_ref()[..], &signature, &message).unwrap());
	}

	#[test]
	fn public_addition() {
		let pk1 = [126, 60, 36, 91, 73, 177, 194, 111, 11, 3, 99, 246, 204, 86, 122, 109, 85, 28, 43, 169, 243, 35, 76, 152, 90, 76, 241, 17, 108, 232, 215, 115, 15, 19, 23, 164, 151, 43, 28, 44, 59, 141, 167, 134, 112, 105, 251, 15, 193, 183, 224, 238, 154, 204, 230, 163, 216, 235, 112, 77, 239, 98, 135, 132];
		let pk2 = [40, 127, 167, 223, 38, 53, 6, 223, 67, 83, 204, 60, 226, 227, 107, 231, 172, 34, 3, 187, 79, 112, 167, 0, 217, 118, 69, 218, 189, 208, 150, 190, 54, 186, 220, 95, 80, 220, 183, 202, 117, 160, 18, 84, 245, 181, 23, 32, 51, 73, 178, 173, 92, 118, 92, 122, 83, 49, 54, 195, 194, 16, 229, 39];
		let pub1 = public_from_slice(&pk1[..]).unwrap();
		let pub2 = public_from_slice(&pk2[..]).unwrap();
		let res = public_add(pub1, &pub2).unwrap();

		assert_eq!(&public_to_vec(&res).as_ref()[..], &[101, 166, 20, 152, 34, 76, 121, 113, 139, 80, 13, 92, 122, 96, 38, 194, 205, 149, 93, 19, 147, 132, 195, 173, 42, 86, 26, 221, 170, 127, 180, 168, 145, 21, 75, 45, 248, 90, 114, 118, 62, 196, 194, 143, 245, 204, 184, 16, 175, 202, 175, 228, 207, 112, 219, 94, 237, 75, 105, 186, 56, 102, 46, 147][..]);
	}

	#[test]
	fn public_multiplication() {
		let pk = [126, 60, 36, 91, 73, 177, 194, 111, 11, 3, 99, 246, 204, 86, 122, 109, 85, 28, 43, 169, 243, 35, 76, 152, 90, 76, 241, 17, 108, 232, 215, 115, 15, 19, 23, 164, 151, 43, 28, 44, 59, 141, 167, 134, 112, 105, 251, 15, 193, 183, 224, 238, 154, 204, 230, 163, 216, 235, 112, 77, 239, 98, 135, 132];
		let sk = [213, 68, 220, 102, 106, 158, 142, 136, 198, 84, 32, 178, 49, 72, 194, 143, 116, 165, 155, 122, 20, 120, 169, 29, 129, 128, 206, 190, 48, 122, 97, 52];
		let pubk = public_from_slice(&pk[..]).unwrap();
		let sec = secret_from_slice(&sk[..]).unwrap();
		let res = public_mul(pubk, &sec).unwrap();

		assert_eq!(&public_to_vec(&res).as_ref()[..], &[98, 132, 11, 170, 93, 231, 41, 185, 180, 151, 185, 130, 77, 251, 41, 169, 160, 84, 133, 19, 82, 190, 137, 82, 0, 214, 148, 120, 165, 184, 17, 21, 237, 184, 119, 174, 13, 77, 50, 251, 16, 17, 197, 74, 232, 55, 142, 220, 27, 152, 4, 52, 69, 14, 76, 8, 156, 82, 0, 193, 179, 65, 63, 106][..]);
	}


	#[test]
	fn public_addition_is_commutative() {
		let mut osrng = OsRng::new().expect("test");
		let (_, public1) = generate_keypair(&mut osrng);
		let (_, public2) = generate_keypair(&mut osrng);

		let left = public_add(public1.clone(), &public2).unwrap();

		let right = public_add(public2.clone(), &public1).unwrap();

		assert_eq!(left, right);
	}

	#[test]
	fn public_addition_is_reversible_with_subtraction() {
		let mut osrng = OsRng::new().expect("test");
		let (_, public1) = generate_keypair(&mut osrng);
		let (_, public2) = generate_keypair(&mut osrng);

		let sum = public_add(public1.clone(), &public2).unwrap();
		let op = public_mul(public2.clone(), minus_one_key()).unwrap();
		let sum = public_add(sum, &op).unwrap();

		assert_eq!(sum, public1);
	}


	#[test]
	fn multiplicating_secret_inversion_with_secret_gives_one() {
		let mut osrng = OsRng::new().expect("test");
		let (secret, _) = generate_keypair(&mut osrng);
		let inversion = secret_inv(secret.clone()).unwrap();
		let inversion = secret_mul(inversion, &secret).unwrap();
		assert_eq!(inversion, *one_key());
	}

	#[test]
	fn secret_inversion_is_reversible_with_inversion() {
		let mut osrng = OsRng::new().expect("test");
		let (secret, _) = generate_keypair(&mut osrng);
		let inversion = secret_inv(secret.clone()).unwrap();
		let inversion = secret_inv(inversion).unwrap();
		assert_eq!(inversion, secret);
	}

}

