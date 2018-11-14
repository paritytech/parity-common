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

#[cfg(test)]
::tests_hmac!();
