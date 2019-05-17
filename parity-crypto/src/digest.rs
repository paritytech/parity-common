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

use rripemd160;
use rsha2;
use std::marker::PhantomData;
use std::ops::Deref;
use rdigest::generic_array::GenericArray;
use rdigest::generic_array::typenum::{U20, U32, U64};
use rsha2::Digest as RDigest;

/// The message digest.
pub struct Digest<T>(InnerDigest, PhantomData<T>);

enum InnerDigest {
	Sha256(GenericArray<u8, U32>),
	Sha512(GenericArray<u8, U64>),
	Ripemd160(GenericArray<u8, U20>),
}

impl<T> Deref for Digest<T> {
	type Target = [u8];
	fn deref(&self) -> &Self::Target {
		match self.0 {
			InnerDigest::Sha256(ref d) => &d[..],
			InnerDigest::Sha512(ref d) => &d[..],
			InnerDigest::Ripemd160(ref d) => &d[..],
		}
	}
}

/// Single-step sha256 digest computation.
pub fn sha256(data: &[u8]) -> Digest<Sha256> {
	let mut hasher = Hasher::sha256();
	hasher.update(data);
	hasher.finish()
}

/// Single-step sha512 digest computation.
pub fn sha512(data: &[u8]) -> Digest<Sha512> {
	let mut hasher = Hasher::sha512();
	hasher.update(data);
	hasher.finish()
}

/// Single-step ripemd160 digest computation.
pub fn ripemd160(data: &[u8]) -> Digest<Ripemd160> {
	let mut hasher = Hasher::ripemd160();
	hasher.update(data);
	hasher.finish()
}

#[derive(Debug)]
pub enum Sha256 {}
#[derive(Debug)]
pub enum Sha512 {}
#[derive(Debug)]
pub enum Ripemd160 {}

/// Stateful digest computation.
pub struct Hasher<T>(Inner, PhantomData<T>);

enum Inner {
	Sha256(rsha2::Sha256),
	Sha512(rsha2::Sha512),
	Ripemd160(rripemd160::Ripemd160)
}

impl Hasher<Sha256> {
	pub fn sha256() -> Hasher<Sha256> {
		Hasher(Inner::Sha256(rsha2::Sha256::default()), PhantomData)
	}
}

impl Hasher<Sha512> {
	pub fn sha512() -> Hasher<Sha512> {
		Hasher(Inner::Sha512(rsha2::Sha512::default()), PhantomData)
	}
}

impl Hasher<Ripemd160> {
	pub fn ripemd160() -> Hasher<Ripemd160> {
		Hasher(Inner::Ripemd160(rripemd160::Ripemd160::default()), PhantomData)
	}
}

impl<T> Hasher<T> {
	pub fn update(&mut self, data: &[u8]) {
		match self.0 {
			Inner::Sha256(ref mut ctx) => {
				ctx.input(data)
			},
			Inner::Sha512(ref mut ctx) => {
				ctx.input(data)
			},
			Inner::Ripemd160(ref mut ctx) => {
				ctx.input(data)
			}
		}
	}

	pub fn finish(self) -> Digest<T> {
		match self.0 {
			Inner::Sha256(ctx) => {
				Digest(InnerDigest::Sha256(ctx.result()), PhantomData)
			},
			Inner::Sha512(ctx) => {
				Digest(InnerDigest::Sha512(ctx.result()), PhantomData)
			},
			Inner::Ripemd160(ctx) => {
				Digest(InnerDigest::Ripemd160(ctx.result()), PhantomData)
			}
		}
	}
}
