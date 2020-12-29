// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::marker::PhantomData;
use std::ops::Deref;

use digest::generic_array::{
	typenum::{U32, U64},
	GenericArray,
};
use hmac::{Hmac, Mac as _, NewMac as _};
use zeroize::Zeroize;

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
struct DisposableBox(Box<[u8]>);

impl std::fmt::Debug for DisposableBox {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", &self.0.as_ref())
	}
}

impl DisposableBox {
	fn from_slice(data: &[u8]) -> Self {
		Self(data.to_vec().into_boxed_slice())
	}
}

impl Drop for DisposableBox {
	fn drop(&mut self) {
		self.0.zeroize()
	}
}

#[derive(Debug, PartialEq)]
enum KeyInner {
	Sha256(DisposableBox),
	Sha512(DisposableBox),
}

impl SigKey<Sha256> {
	pub fn sha256(key: &[u8]) -> SigKey<Sha256> {
		SigKey(KeyInner::Sha256(DisposableBox::from_slice(key)), PhantomData)
	}
}

impl SigKey<Sha512> {
	pub fn sha512(key: &[u8]) -> SigKey<Sha512> {
		SigKey(KeyInner::Sha512(DisposableBox::from_slice(key)), PhantomData)
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
			KeyInner::Sha256(key_bytes) => Signer(
				SignerInner::Sha256(Hmac::<sha2::Sha256>::new_varkey(&key_bytes.0).expect("always returns Ok; qed")),
				PhantomData,
			),
			KeyInner::Sha512(key_bytes) => Signer(
				SignerInner::Sha512(Hmac::<sha2::Sha512>::new_varkey(&key_bytes.0).expect("always returns Ok; qed")),
				PhantomData,
			),
		}
	}

	pub fn update(&mut self, data: &[u8]) {
		match &mut self.0 {
			SignerInner::Sha256(hmac) => hmac.update(data),
			SignerInner::Sha512(hmac) => hmac.update(data),
		}
	}

	pub fn sign(self) -> Signature<T> {
		match self.0 {
			SignerInner::Sha256(hmac) => Signature(HashInner::Sha256(hmac.finalize().into_bytes()), PhantomData),
			SignerInner::Sha512(hmac) => Signature(HashInner::Sha512(hmac.finalize().into_bytes()), PhantomData),
		}
	}
}

/// HMAC signature verification key.
pub struct VerifyKey<T>(KeyInner, PhantomData<T>);

impl VerifyKey<Sha256> {
	pub fn sha256(key: &[u8]) -> VerifyKey<Sha256> {
		VerifyKey(KeyInner::Sha256(DisposableBox::from_slice(key)), PhantomData)
	}
}

impl VerifyKey<Sha512> {
	pub fn sha512(key: &[u8]) -> VerifyKey<Sha512> {
		VerifyKey(KeyInner::Sha512(DisposableBox::from_slice(key)), PhantomData)
	}
}

/// Verify HMAC signature of `data`.
pub fn verify<T>(key: &VerifyKey<T>, data: &[u8], sig: &[u8]) -> bool {
	match &key.0 {
		KeyInner::Sha256(key_bytes) => {
			let mut ctx = Hmac::<sha2::Sha256>::new_varkey(&key_bytes.0).expect("always returns Ok; qed");
			ctx.update(data);
			ctx.verify(sig).is_ok()
		}
		KeyInner::Sha512(key_bytes) => {
			let mut ctx = Hmac::<sha2::Sha512>::new_varkey(&key_bytes.0).expect("always returns Ok; qed");
			ctx.update(data);
			ctx.verify(sig).is_ok()
		}
	}
}

#[cfg(test)]
mod test;
