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

//! asymetric trait

extern crate rand;

use ::error::Error;
use self::rand::Rng;

/// Trait for asymetric crypto
pub trait Asym {

	/// Signature expected size in bytes
	const SIGN_SIZE: usize;

	/// Public key expected size in bytes
	const PUB_SIZE: usize;

	/// Private key expected size in bytes
	const SECRET_SIZE: usize;

	/// Size of secure random input require
	/// to generate a keypair
	const KEYPAIR_INPUT_SIZE: usize;

	/// Associated type for Public Key
	type PublicKey: PublicKey;

	/// Associated type for Private key
	type SecretKey: SecretKey;

	/// Recover a public key from a signature over a message
	/// This function could move to a more specific trait in the future
	fn recover(signature: &[u8], message: &[u8]) -> Result<Self::PublicKey, Error>;

	/// Generate a key pair from a random function.
	#[deprecated]
	fn generate_keypair(r: &mut impl Rng) -> (Self::SecretKey, Self::PublicKey);

	/// Generate a keypair from a random input
	fn keypair_from_slice(bytes: &[u8]) -> Result<(Self::SecretKey, Self::PublicKey), Error>;

	/// Generate a public key from a secret key
	/// This function could move to a more specific trait in the future
	fn public_from_secret(s: &Self::SecretKey) -> Result<Self::PublicKey, Error>;

	/// Instantiate a public key from its byte representation
	fn public_from_slice(bytes: &[u8]) -> Result<Self::PublicKey, Error>;

	/// Instantiate a private key from its byte representation
	fn secret_from_slice(bytes: &[u8]) -> Result<Self::SecretKey, Error>;

}


pub trait FixAsymSharedSecret: SecretKey {
	type Other;
	type Result: AsRef<[u8]>;

	// TODO replace by Result<impl AsRef<[u8]>> when supported
	fn shared_secret(&self, publ: &Self::Other) -> Result<Self::Result, Error>;

}

/// Some Finite field arithmetic primitives.
pub trait FiniteField: Asym {

	fn public_mul(_pub_key: &mut Self::PublicKey, _sec_key: &Self::SecretKey) -> Result<(), Error>;

	fn public_add(_pub_key: &mut Self::PublicKey, _other_public: &Self::PublicKey) -> Result<(), Error>;

	fn secret_mul(_sec_key: &mut Self::SecretKey, _other_secret: &Self::SecretKey) -> Result<(), Error>;

	fn secret_add(_sec_key: &mut Self::SecretKey, _other_secret: &Self::SecretKey) -> Result<(), Error>;

	fn secret_inv(_sec_key: &mut Self::SecretKey) -> Result<(), Error>;

	fn zero_key() -> &'static Self::SecretKey;

	fn one_key() -> &'static Self::SecretKey;

	fn minus_one_key() -> &'static Self::SecretKey;

	// those fn should not be exposed and function using them instead
	fn generator_x() -> &'static[u8];
	fn generator_y() -> &'static[u8];
	fn curve_order() -> &'static[u8];
}


/// PublicKey.
/// Contraint AsRef<[u8]>` is not memory efficient for ffi.
/// Keeping in mind that the trait is here to make thing easier
/// in a parity context. We assert that for use cases such as parity ethereum it is very usefull.
/// In the future a switch to having only a function returning `impl AsRef<[u8]>`
/// could be done but at the time it involves moving a huge amount of logic to this crate. 
pub trait PublicKey: Sized + Eq + PartialEq + Clone + AsRef<[u8]> {
	type VecRepr: AsRef<[u8]>;

	/// Should move to another trait.
	fn to_compressed_vec(&self) -> Self::VecRepr;

	/// Compatibility, this should disappear, public key should always be valid.
	fn is_valid(&self) -> bool;
	
	fn verify(&self, signature: &[u8], message: &[u8]) -> Result<bool, Error>;

}

/// SecretKey (Private key).
pub trait SecretKey: Sized + Eq + PartialEq + Clone + AsRef<[u8]> {

	fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Error>;
	
}


/// This macro is callable only if the parent trait
/// contains a type `AsymTest` that implements `asym`.
#[cfg(test)]
#[macro_export]
macro_rules! tests_asym {
		() => {

#[cfg(test)]
mod tests {
	extern crate rand;
	use super::AsymTest;
	use ::traits::asym::*;
	use self::rand::OsRng;
	use self::rand::Rng;

	#[test]
	fn sign_val() {
		let sk = [213, 68, 220, 102, 106, 158, 142, 136, 198, 84, 32, 178, 49, 72, 194, 143, 116, 165, 155, 122, 20, 120, 169, 29, 129, 128, 206, 190, 48, 122, 97, 52];
		let sec = AsymTest::secret_from_slice(&sk).unwrap();
		let message = vec![2;32];
		let signature = sec.sign(&message).unwrap();
		assert_eq!(&signature[..], &[88, 96, 150, 252, 139, 37, 138, 196, 9, 30, 22, 98, 125, 20, 223, 16, 221, 46, 42, 225, 164, 71, 221, 37, 81, 9, 58, 3, 31, 245, 121, 110, 0, 248, 154, 65, 12, 193, 151, 151, 236, 69, 230, 56, 39, 161, 124, 1, 30, 20, 130, 5, 174, 75, 254, 199, 5, 119, 39, 223, 20, 116, 11, 229, 0][..]);
	}

	#[test]
	fn sign_and_recover_public() {
		let mut osrng = OsRng::new().expect("test");
		let mut sec_buf = vec![0; AsymTest::SECRET_SIZE];
		osrng.fill_bytes(&mut sec_buf[..]);
		let (secret, public) = AsymTest::keypair_from_slice(&mut sec_buf).unwrap();
		let message = vec![2;32];
		let signature = secret.sign(&message).unwrap();
		assert_eq!(public, AsymTest::recover(&signature, &message).unwrap());
	}

	#[test]
	fn sign_and_verify_public() {
		let mut osrng = OsRng::new().expect("test");
		let mut sec_buf = vec![0; AsymTest::SECRET_SIZE];
		osrng.fill_bytes(&mut sec_buf[..]);
		let (secret, public) = AsymTest::keypair_from_slice(&mut sec_buf).unwrap();
		let message = vec![0;32];
		let signature = secret.sign(&message).unwrap();
		assert!(public.verify(&signature, &message).unwrap());
	}

	#[test]
	fn public_addition() {
		let pk1 = [126, 60, 36, 91, 73, 177, 194, 111, 11, 3, 99, 246, 204, 86, 122, 109, 85, 28, 43, 169, 243, 35, 76, 152, 90, 76, 241, 17, 108, 232, 215, 115, 15, 19, 23, 164, 151, 43, 28, 44, 59, 141, 167, 134, 112, 105, 251, 15, 193, 183, 224, 238, 154, 204, 230, 163, 216, 235, 112, 77, 239, 98, 135, 132];
		let pk2 = [40, 127, 167, 223, 38, 53, 6, 223, 67, 83, 204, 60, 226, 227, 107, 231, 172, 34, 3, 187, 79, 112, 167, 0, 217, 118, 69, 218, 189, 208, 150, 190, 54, 186, 220, 95, 80, 220, 183, 202, 117, 160, 18, 84, 245, 181, 23, 32, 51, 73, 178, 173, 92, 118, 92, 122, 83, 49, 54, 195, 194, 16, 229, 39];
		let mut pub1 = AsymTest::public_from_slice(&pk1[..]).unwrap();
		let pub2 = AsymTest::public_from_slice(&pk2[..]).unwrap();
		AsymTest::public_add(&mut pub1, &pub2).unwrap();

		assert_eq!(&pub1.as_ref()[..], &[101, 166, 20, 152, 34, 76, 121, 113, 139, 80, 13, 92, 122, 96, 38, 194, 205, 149, 93, 19, 147, 132, 195, 173, 42, 86, 26, 221, 170, 127, 180, 168, 145, 21, 75, 45, 248, 90, 114, 118, 62, 196, 194, 143, 245, 204, 184, 16, 175, 202, 175, 228, 207, 112, 219, 94, 237, 75, 105, 186, 56, 102, 46, 147][..]);
	}

	#[test]
	fn public_multiplication() {
		let pk = [126, 60, 36, 91, 73, 177, 194, 111, 11, 3, 99, 246, 204, 86, 122, 109, 85, 28, 43, 169, 243, 35, 76, 152, 90, 76, 241, 17, 108, 232, 215, 115, 15, 19, 23, 164, 151, 43, 28, 44, 59, 141, 167, 134, 112, 105, 251, 15, 193, 183, 224, 238, 154, 204, 230, 163, 216, 235, 112, 77, 239, 98, 135, 132];
		let sk = [213, 68, 220, 102, 106, 158, 142, 136, 198, 84, 32, 178, 49, 72, 194, 143, 116, 165, 155, 122, 20, 120, 169, 29, 129, 128, 206, 190, 48, 122, 97, 52];
		let mut pubk = AsymTest::public_from_slice(&pk[..]).unwrap();
		let sec = AsymTest::secret_from_slice(&sk[..]).unwrap();
		AsymTest::public_mul(&mut pubk, &sec).unwrap();

		assert_eq!(&pubk.as_ref()[..], &[98, 132, 11, 170, 93, 231, 41, 185, 180, 151, 185, 130, 77, 251, 41, 169, 160, 84, 133, 19, 82, 190, 137, 82, 0, 214, 148, 120, 165, 184, 17, 21, 237, 184, 119, 174, 13, 77, 50, 251, 16, 17, 197, 74, 232, 55, 142, 220, 27, 152, 4, 52, 69, 14, 76, 8, 156, 82, 0, 193, 179, 65, 63, 106][..]);
	}


	#[test]
	fn public_addition_is_commutative() {
		let mut osrng = OsRng::new().expect("test");
		let mut sec_buf = vec![0; AsymTest::SECRET_SIZE];
		osrng.fill_bytes(&mut sec_buf[..]);
		let (_, public1) = AsymTest::keypair_from_slice(&mut sec_buf).unwrap();
		osrng.fill_bytes(&mut sec_buf[..]);
		let (_, public2) = AsymTest::keypair_from_slice(&mut sec_buf).unwrap();

		let mut left = public1.clone();
		AsymTest::public_add(&mut left, &public2).unwrap();

		let mut right = public2.clone();
		AsymTest::public_add(&mut right, &public1).unwrap();

		assert_eq!(left, right);
	}

	#[test]
	fn public_addition_is_reversible_with_subtraction() {
		let mut osrng = OsRng::new().expect("test");
		let mut sec_buf = vec![0; AsymTest::SECRET_SIZE];
		osrng.fill_bytes(&mut sec_buf[..]);
		let (_, public1) = AsymTest::keypair_from_slice(&mut sec_buf).unwrap();
		osrng.fill_bytes(&mut sec_buf[..]);
		let (_, public2) = AsymTest::keypair_from_slice(&mut sec_buf).unwrap();

		let mut sum = public1.clone();
		AsymTest::public_add(&mut sum, &public2).unwrap();
		let mut op = public2.clone();
		AsymTest::public_mul(&mut op, AsymTest::minus_one_key()).unwrap();
		AsymTest::public_add(&mut sum, &op).unwrap();

		assert_eq!(sum, public1);
	}


	#[test]
	fn multiplicating_secret_inversion_with_secret_gives_one() {
		let mut osrng = OsRng::new().expect("test");
		let mut sec_buf = vec![0; AsymTest::SECRET_SIZE];
		osrng.fill_bytes(&mut sec_buf[..]);
		let (secret, _) = AsymTest::keypair_from_slice(&mut sec_buf).unwrap();

		let mut inversion = secret.clone();
		AsymTest::secret_inv(&mut inversion).unwrap();
		AsymTest::secret_mul(&mut inversion, &secret).unwrap();
		assert_eq!(inversion, *AsymTest::one_key());
	}

	#[test]
	fn secret_inversion_is_reversible_with_inversion() {
		let mut osrng = OsRng::new().expect("test");
		let mut sec_buf = vec![0; AsymTest::SECRET_SIZE];
		osrng.fill_bytes(&mut sec_buf[..]);
		let (secret, _) = AsymTest::keypair_from_slice(&mut sec_buf).unwrap();
		let mut inversion = secret.clone();
		AsymTest::secret_inv(&mut inversion).unwrap();
		AsymTest::secret_inv(&mut inversion).unwrap();
		assert_eq!(inversion, secret);
	}

	#[test]
	fn serialize_keys() {
		let pk = [126, 60, 36, 91, 73, 177, 194, 111, 11, 3, 99, 246, 204, 86, 122, 109, 85, 28, 43, 169, 243, 35, 76, 152, 90, 76, 241, 17, 108, 232, 215, 115, 15, 19, 23, 164, 151, 43, 28, 44, 59, 141, 167, 134, 112, 105, 251, 15, 193, 183, 224, 238, 154, 204, 230, 163, 216, 235, 112, 77, 239, 98, 135, 132];
		let sk = [213, 68, 220, 102, 106, 158, 142, 136, 198, 84, 32, 178, 49, 72, 194, 143, 116, 165, 155, 122, 20, 120, 169, 29, 129, 128, 206, 190, 48, 122, 97, 52];
		let ck = [2, 126, 60, 36, 91, 73, 177, 194, 111, 11, 3, 99, 246, 204, 86, 122, 109, 85, 28, 43, 169, 243, 35, 76, 152, 90, 76, 241, 17, 108, 232, 215, 115];
		let pubk = AsymTest::public_from_slice(&pk[..]).unwrap();
		let sec = AsymTest::secret_from_slice(&sk[..]).unwrap();
		assert!(pubk.as_ref() == &pk[..]);
		assert!(sec.as_ref() == &sk[..]);
		assert!(AsRef::<[u8]>::as_ref(&pubk.to_compressed_vec()) == &ck[..]);
	}
}

}
}
