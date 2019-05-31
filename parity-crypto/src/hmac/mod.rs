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

use std::marker::PhantomData;
use std::ops::Deref;

use digest::generic_array::{GenericArray, typenum::{U32, U64}};
use hmac::{Hmac, Mac as _};
use memzero::Memzero;

use crate::digest::{Sha256, Sha512};

/// HMAC signature.
#[derive(Debug)]
pub struct Signature<T>(HashInner, PhantomData<T>);

#[derive(Debug)]
enum HashInner {
	Sha256(GenericArray<u8, U32>),
	Sha512(GenericArray<u8, U64>),
}

impl<T> Deref for Signature<T> {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		match &self.0 {
			HashInner::Sha256(a) => a.as_slice(),
			HashInner::Sha512(a) => a.as_slice(),
		}
	}
}

/// HMAC signing key.
pub struct SigKey<T>(KeyInner, PhantomData<T>);

#[derive(PartialEq)]
// Using `Box[u8]` guarantees no reallocation can happen
struct DisposableBox(Memzero<Box<[u8]>>);

impl std::fmt::Debug for DisposableBox {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", &self.0.as_ref())
	}
}

impl DisposableBox {
	fn from_slice(data: &[u8]) -> Self {
		Self(Memzero::from(data.to_vec().into_boxed_slice()))
	}
}

#[derive(Debug, PartialEq)]
enum KeyInner {
	Sha256(DisposableBox),
	Sha512(DisposableBox),
}

impl SigKey<Sha256> {
	pub fn sha256(key: &[u8]) -> SigKey<Sha256> {
		SigKey(
			KeyInner::Sha256(DisposableBox::from_slice(key)),
			PhantomData
		)
	}
}

impl SigKey<Sha512> {
	pub fn sha512(key: &[u8]) -> SigKey<Sha512> {
		SigKey(
			KeyInner::Sha512(DisposableBox::from_slice(key)),
			PhantomData
		)
	}
}

/// Compute HMAC signature of `data`.
pub fn sign<T>(k: &SigKey<T>, data: &[u8]) -> Signature<T> {
	let mut signer = Signer::with(k);
	signer.update(data);
	signer.sign()
}

/// Stateful HMAC computation.
pub struct Signer<T>(SignerInner, PhantomData<T>);

enum SignerInner {
	Sha256(Hmac<sha2::Sha256>),
	Sha512(Hmac<sha2::Sha512>),
}

impl<T> Signer<T> {
	pub fn with(key: &SigKey<T>) -> Signer<T> {
		match &key.0 {
			KeyInner::Sha256(key_bytes) => {
				Signer(
					SignerInner::Sha256(
						Hmac::<sha2::Sha256>::new_varkey(&key_bytes.0)
							.expect("always returns Ok; qed")
					),
					PhantomData
				)
			},
			KeyInner::Sha512(key_bytes) => {
				Signer(
					SignerInner::Sha512(
						Hmac::<sha2::Sha512>::new_varkey(&key_bytes.0)
							.expect("always returns Ok; qed")
					), PhantomData
				)
			},
		}
	}

	pub fn update(&mut self, data: &[u8]) {
		match &mut self.0 {
			SignerInner::Sha256(hmac) => hmac.input(data),
			SignerInner::Sha512(hmac) => hmac.input(data),
		}
	}

	pub fn sign(self) -> Signature<T> {
		match self.0 {
			SignerInner::Sha256(hmac) => Signature(HashInner::Sha256(hmac.result().code()), PhantomData),
			SignerInner::Sha512(hmac) => Signature(HashInner::Sha512(hmac.result().code()), PhantomData),
		}
	}
}

/// HMAC signature verification key.
pub struct VerifyKey<T>(KeyInner, PhantomData<T>);

impl VerifyKey<Sha256> {
	pub fn sha256(key: &[u8]) -> VerifyKey<Sha256> {
		VerifyKey(
			KeyInner::Sha256(DisposableBox::from_slice(key)),
			PhantomData
		)
	}
}

impl VerifyKey<Sha512> {
	pub fn sha512(key: &[u8]) -> VerifyKey<Sha512> {
		VerifyKey(
			KeyInner::Sha512(DisposableBox::from_slice(key)),
			PhantomData
		)
	}
}

/// Verify HMAC signature of `data`.
pub fn verify<T>(key: &VerifyKey<T>, data: &[u8], sig: &[u8]) -> bool {
	match &key.0 {
		KeyInner::Sha256(key_bytes) => {
			let mut ctx = Hmac::<sha2::Sha256>::new_varkey(&key_bytes.0)
				.expect("always returns Ok; qed");
			ctx.input(data);
			ctx.verify(sig).is_ok()
		},
		KeyInner::Sha512(key_bytes) => {
			let mut ctx = Hmac::<sha2::Sha512>::new_varkey(&key_bytes.0)
				.expect("always returns Ok; qed");
			ctx.input(data);
			ctx.verify(sig).is_ok()
		},
	}
}

#[cfg(test)]
mod test;
