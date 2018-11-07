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
extern crate rand;

use self::rand::Rng;


pub use self::secp256k1::{
	Error,
	PublicKey,
	SecretKey,
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

pub const SECRET_KEY_SIZE: usize = 32;

const MINUS_ONE_BYTES: [u8;32] = [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 254, 186, 174, 220, 230, 175, 72, 160, 59, 191, 210, 94, 140, 208, 54, 65, 64];

const ONE_BYTES: [u8;32] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];


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
	static ref MINUS_ONE_KEY: SecretKey = SecretKey::parse(&MINUS_ONE_BYTES).expect("static; qed");
	static ref ONE_KEY: SecretKey = SecretKey::parse(&ONE_BYTES).expect("static; qed");
}

pub fn one_key() -> &'static SecretKey {
	&ONE_KEY
}

pub fn minus_one_key() -> &'static SecretKey {
	&MINUS_ONE_KEY
}


/// secret size 32, message size 32
pub fn sign(secret: &[u8], message: &[u8]) -> Result<[u8;65], Error> {
	let mut buf = [0;32];
	buf.copy_from_slice(&message[..]); // panic on incorrect message size
	let message = Message::parse(&buf);
	buf.copy_from_slice(&secret[..]); // panic on incorrect secret size
	let seckey = SecretKey::parse(&buf)?;
	let (sig, rec_id) = secp256k1::sign(&message, &seckey)?;
	let mut data_arr = [0; 65];

	data_arr[0..64].copy_from_slice(&sig.serialize());
	data_arr[64] = rec_id.serialize();
	Ok(data_arr)
}

/// public size 65, signature size 65, message size 32
pub fn verify_public(public: &[u8], signature: &[u8], message: &[u8]) -> Result<bool, Error> {

	let mut buf = [0;32];
	buf.copy_from_slice(&message[..]); // panic on incorrect message size
	let message = Message::parse(&buf);
	let mut buf = [4;65];
	buf[1..65].copy_from_slice(&public[..]); // panic on incorrect public size
	let pubkey = PublicKey::parse(&buf)?;

	let mut buf = [0;64];
	buf.copy_from_slice(&signature[..64]); // panic on incorrect signature size
	let signature = Signature::parse(&buf);

	Ok(secp256k1::verify(&message, &signature, &pubkey))
}

/// signature 65, message 32, return publickey 64 bit (no start 4)
pub fn recover(signature: &[u8], message: &[u8]) -> Result<[u8;64], Error> {
	let mut buf = [0;32];
	buf.copy_from_slice(&message[..]); // panic on incorrect message size
	let message = Message::parse(&buf);
	let mut buf = [0;64];
	buf.copy_from_slice(&signature[..64]); // panic on incorrect signature size
	let recovery_id = RecoveryId::parse(signature[64])?; 
	let signature = Signature::parse(&buf);
	let public_key = secp256k1::recover(&message, &signature, &recovery_id)?;

	let mut res = [0;64];
	res.copy_from_slice(&public_key.serialize()[1..65]);

	Ok(res)
}
/// random secret key for rand 0.5
pub fn random_sec<R: Rng>(rng: &mut R) -> SecretKey {
	loop {
		let mut ret = [0u8; 32];
		rng.fill_bytes(&mut ret);

		match SecretKey::parse(&ret) {
			Ok(key) => return key,
			Err(_) => (),
		}
	}
}

pub fn generate_keypair(r: &mut impl Rng) -> (SecretKey, PublicKey) {
	let secret_key = random_sec(r);
	let public_key = PublicKey::from_secret_key(&secret_key);
	(secret_key, public_key)
}

/// warning this returns 64 byte vec (we skip the first byte of 65 byte more standard
/// representation)
pub fn public_to_vec(p: &PublicKey) -> impl AsRef<[u8]> {
	let a_vec = p.serialize();
	a_vec[1..65].to_vec()
}

pub fn public_is_valid(p: &PublicKey) -> bool {
	// Check from other implementation only look for a non zero value in fields
	// here we can
	let aff: Affine = p.clone().into();
	aff.is_valid_var()
}

/// ret size 33
pub fn public_to_compressed_vec(p: &PublicKey) -> impl AsRef<[u8]> {
	p.serialize_compressed().to_vec()
}

/// only for test (or make the result erasable)
pub fn secret_to_vec(p: &SecretKey) -> impl AsRef<[u8]> {
	p.serialize()
}

/// 32 sized slice
pub fn secret_from_slice(secret: &[u8]) -> Result<SecretKey, Error> {
	let mut buf = [0;32];
	buf.copy_from_slice(&secret[..]); // panic on incorrect secret size
	SecretKey::parse(&buf)
}

pub fn shared_secret(publ: &PublicKey, sec: &SecretKey) -> Result<impl AsRef<[u8]>, Error> {
	secp256k1::SharedSecret::new(publ, sec)
}

/// using a shortened 64bit public key as input
pub fn public_from_slice(public_sec_raw: &[u8]) -> Result<PublicKey, Error> {
	let mut buf = [4;65];
	buf[1..65].copy_from_slice(&public_sec_raw[..]); // panic on incorrect public size
	PublicKey::parse(&buf)
}

pub fn public_from_secret(s: &SecretKey) -> Result<PublicKey, Error> {
	Ok(PublicKey::from_secret_key(&s))
}


fn aff_to_public(aff_pub: &mut Affine) -> Result<PublicKey, Error> {
	let mut buff = [4;65];
	let mut buff2 = [0;32];
	aff_pub.x.normalize();
	aff_pub.x.fill_b32(&mut buff2);
	buff[1..33].copy_from_slice(&buff2[..]);
	aff_pub.y.normalize();
	aff_pub.y.fill_b32(&mut buff2);
	buff[33..65].copy_from_slice(&buff2[..]);
	PublicKey::parse(&buff)

}
pub fn public_add(pub_key: PublicKey, other_public: &PublicKey) -> Result<PublicKey, Error> {
	let mut aff_pub: Affine = pub_key.into();
	let mut aff_pub_j = Jacobian::default();
	aff_pub_j.set_ge(&aff_pub);
	let aff_pub_other: Affine = other_public.clone().into();
	let res_j = aff_pub_j.add_ge(&aff_pub_other);
	aff_pub.set_gej(&res_j);
	aff_to_public(&mut aff_pub)
}

struct SecretScalar(pub Scalar);

impl Drop for SecretScalar {
	fn drop(&mut self) {
		self.0.clear();
	}
}

pub fn public_mul(pub_key: PublicKey, sec_key: &SecretKey) -> Result<PublicKey, Error> {
/*	let mut sec_scal = Scalar::default();
	sec_scal.set_b32(&sec_key.serialize());*/

	let sec_scal = SecretScalar(sec_key.clone().into());
	let mut pub_aff: Affine = pub_key.into();
	let mut pub_jac = Jacobian::default();
	pub_jac.set_ge(&pub_aff);

	//ECMULT_GEN_CONTEXT.ecmult_gen(&mut pub_jac, &sec_scal);
	//pub_aff.set_gej(&pub_jac);
	let mut zero = Scalar::default();
	zero.set_int(0);
	let mut res = Jacobian::default();
	ECMULT_CONTEXT.ecmult(&mut res, &pub_jac, &sec_scal.0, &zero);
	pub_aff.set_gej(&res);
	aff_to_public(&mut pub_aff)
}

/* private inner method but this would avoid a scalar instantiation
fn mul_in_place_scalar(a: &mut Scalar, b: &Scalar) {
	let mut l = [0u32; 16];
	a.mul_512(b, &mut l);
	a.reduce_512(&l);
}
*/

pub fn secret_mul(sec_key: SecretKey, other_sec_key: &SecretKey) -> Result<SecretKey, Error> {
	let sec_scal = SecretScalar(sec_key.clone().into());
	let other_sec_scal = SecretScalar(other_sec_key.clone().into());
	// we could use * operator instead.
	let mut res = SecretScalar(Scalar::default());
	res.0.mul_in_place(&sec_scal.0, &other_sec_scal.0);
	SecretKey::parse(&res.0.b32())
}

pub fn secret_add(sec_key: SecretKey, other_sec_key: &SecretKey) -> Result<SecretKey, Error> {
	let sec_scal = SecretScalar(sec_key.clone().into());
	let other_sec_scal = SecretScalar(other_sec_key.clone().into());
	// we could use + operator instead.
	let mut res = SecretScalar(Scalar::default());
	res.0.add_in_place(&sec_scal.0, &other_sec_scal.0);
	SecretKey::parse(&res.0.b32())
}

pub fn secret_inv(sec_key: SecretKey) -> Result<SecretKey, Error> {
	let sec_scal = SecretScalar(sec_key.clone().into());
	let mut res = SecretScalar(Scalar::default());
	res.0.inv_in_place(&sec_scal.0);
	SecretKey::parse(&res.0.b32())
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
