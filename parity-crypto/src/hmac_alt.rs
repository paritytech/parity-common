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
extern crate hmac;
use rdigest::generic_array::{GenericArray};
use rsha2::Sha256;
use rsha2::Sha512;
use std::marker::PhantomData;
use std::ops::Deref;
use self::hmac::{Hmac, Mac};

/// HMAC signature. Note the public interface of this module is a bit awkward for RustCrypto.
pub struct Signature<T: Mac>(GenericArray<u8, T::OutputSize>, PhantomData<T>);

impl<T: Mac> Deref for Signature<T> {
	type Target = [u8];
	fn deref(&self) -> &Self::Target {
		&self.0[..]
	}
}

/// HMAC signing key.
pub struct SigKey<T: Mac>(GenericArray<u8, T::KeySize>, PhantomData<T>);

impl SigKey<Hmac<Sha256>> {
	pub fn sha256(key: &[u8]) -> SigKey<Hmac<Sha256>> {
		SigKey(GenericArray::clone_from_slice(key), PhantomData)
	}
}

impl SigKey<Hmac<Sha512>> {
	pub fn sha512(key: &[u8]) -> SigKey<Hmac<Sha512>> {
		SigKey(GenericArray::clone_from_slice(key), PhantomData)
	}
}


/// Compute HMAC signature of `data`.
/// TODO consider removal of this method in favor of all stateful
pub fn sign<T: Mac>(k: &SigKey<T>, data: &[u8]) -> Signature<T> {
	let mut sig = Signer::with(k);
	sig.update(data);
	sig.sign()
}

/// Stateful HMAC computation.
pub struct Signer<T: Mac>(T);


impl<T: Mac> Signer<T> {
	pub fn with(key: &SigKey<T>) -> Signer<T> {
		Signer(T::new(&key.0))
	}

	pub fn update(&mut self, data: &[u8]) {
		self.0.input(data)
	}

	pub fn sign(mut self) -> Signature<T> {
		Signature(self.0.result_reset().code(), PhantomData)
	}
}

/// HMAC signature verification key.
pub type VerifyKey<T> = SigKey<T>;

/// Verify HMAC signature of `data`.
pub fn verify<T: Mac>(k: &VerifyKey<T>, data: &[u8], sig: &[u8]) -> bool {
	let mut ver = Signer::with(k);
	ver.update(data);
	ver.0.verify(sig).is_ok()
}

#[test]
fn simple_mac_and_verify() {
	let input = b"Some bytes";
	let big_input = vec![7u8;2000];

	let key1 = vec![3u8;64];
	let key2 = vec![4u8;128];

	let sig_key1 = SigKey::sha256(&key1[..]);
	let sig_key2 = SigKey::sha512(&key2[..]);
	let mut signer1 = Signer::with(&sig_key1);
	let mut signer2 = Signer::with(&sig_key2);

	signer1.update(&input[..]);
	for i in 0 .. big_input.len() / 33 {
		signer2.update(&big_input[i*33..(i+1)*33]);
	}
	signer2.update(&big_input[(big_input.len() / 33)*33..]);
	let sig1 = signer1.sign();
	assert_eq!(&sig1[..], [223, 208, 90, 69, 144, 95, 145, 180, 56, 155, 78, 40, 86, 238, 205, 81, 160, 245, 88, 145, 164, 67, 254, 180, 202, 107, 93, 249, 64, 196, 86, 225]);
	let sig2 = signer2.sign();
	assert_eq!(&sig2[..], &[29, 63, 46, 122, 27, 5, 241, 38, 86, 197, 91, 79, 33, 107, 152, 195, 118, 221, 117, 119, 84, 114, 46, 65, 243, 157, 105, 12, 147, 176, 190, 37, 210, 164, 152, 8, 58, 243, 59, 206, 80, 10, 230, 197, 255, 110, 191, 180, 93, 22, 255, 0, 99, 79, 237, 229, 209, 199, 125, 83, 15, 179, 134, 89][..]);
	
	assert_eq!(&sig1[..], &sign(&sig_key1, &input[..])[..]);
	assert_eq!(&sig2[..], &sign(&sig_key2, &big_input[..])[..]);
}
