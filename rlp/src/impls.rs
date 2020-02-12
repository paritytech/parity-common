// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, string::String, vec::Vec};
use core::iter::{empty, once};
use core::{mem, str};

use crate::error::DecoderError;
use crate::rlpin::Rlp;
use crate::stream::RlpStream;
use crate::traits::{Decodable, Encodable};

pub fn decode_usize(bytes: &[u8]) -> Result<usize, DecoderError> {
	match bytes.len() {
		l if l <= mem::size_of::<usize>() => {
			if bytes[0] == 0 {
				return Err(DecoderError::RlpInvalidIndirection);
			}
			let mut res = 0usize;
			for (i, byte) in bytes.iter().enumerate().take(l) {
				let shift = (l - 1 - i) * 8;
				res += (*byte as usize) << shift;
			}
			Ok(res)
		}
		_ => Err(DecoderError::RlpIsTooBig),
	}
}

impl Encodable for bool {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.encoder().encode_iter(once(if *self { 1u8 } else { 0 }));
	}
}

impl Decodable for bool {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		rlp.decoder().decode_value(|bytes| match bytes.len() {
			0 => Ok(false),
			1 => Ok(bytes[0] != 0),
			_ => Err(DecoderError::RlpIsTooBig),
		})
	}
}

impl<'a> Encodable for &'a [u8] {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.encoder().encode_value(self);
	}
}

impl Encodable for Vec<u8> {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.encoder().encode_value(self);
	}
}

impl Decodable for Vec<u8> {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		rlp.decoder().decode_value(|bytes| Ok(bytes.to_vec()))
	}
}

impl<T> Encodable for Option<T>
where
	T: Encodable,
{
	fn rlp_append(&self, s: &mut RlpStream) {
		match *self {
			None => {
				s.begin_list(0);
			}
			Some(ref value) => {
				s.begin_list(1);
				s.append(value);
			}
		}
	}
}

impl<T> Decodable for Option<T>
where
	T: Decodable,
{
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		let items = rlp.item_count()?;
		match items {
			1 => rlp.val_at(0).map(Some),
			0 => Ok(None),
			_ => Err(DecoderError::RlpIncorrectListLen),
		}
	}
}

impl Encodable for u8 {
	fn rlp_append(&self, s: &mut RlpStream) {
		if *self != 0 {
			s.encoder().encode_iter(once(*self));
		} else {
			s.encoder().encode_iter(empty());
		}
	}
}

impl Decodable for u8 {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		rlp.decoder().decode_value(|bytes| match bytes.len() {
			1 if bytes[0] != 0 => Ok(bytes[0]),
			0 => Ok(0),
			1 => Err(DecoderError::RlpInvalidIndirection),
			_ => Err(DecoderError::RlpIsTooBig),
		})
	}
}

macro_rules! impl_encodable_for_u {
	($name: ident) => {
		impl Encodable for $name {
			fn rlp_append(&self, s: &mut RlpStream) {
				let leading_empty_bytes = self.leading_zeros() as usize / 8;
				let buffer = self.to_be_bytes();
				s.encoder().encode_value(&buffer[leading_empty_bytes..]);
			}
		}
	};
}

macro_rules! impl_decodable_for_u {
	($name: ident) => {
		impl Decodable for $name {
			fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
				rlp.decoder().decode_value(|bytes| match bytes.len() {
					0 | 1 => u8::decode(rlp).map(|v| v as $name),
					l if l <= mem::size_of::<$name>() => {
						if bytes[0] == 0 {
							return Err(DecoderError::RlpInvalidIndirection);
						}
						let mut res = 0 as $name;
						for (i, byte) in bytes.iter().enumerate().take(l) {
							let shift = (l - 1 - i) * 8;
							res += (*byte as $name) << shift;
						}
						Ok(res)
					}
					_ => Err(DecoderError::RlpIsTooBig),
				})
			}
		}
	};
}

impl_encodable_for_u!(u16);
impl_encodable_for_u!(u32);
impl_encodable_for_u!(u64);

impl_decodable_for_u!(u16);
impl_decodable_for_u!(u32);
impl_decodable_for_u!(u64);

impl Encodable for usize {
	fn rlp_append(&self, s: &mut RlpStream) {
		(*self as u64).rlp_append(s);
	}
}

impl Decodable for usize {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		u64::decode(rlp).map(|value| value as usize)
	}
}

impl<'a> Encodable for &'a str {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.encoder().encode_value(self.as_bytes());
	}
}

impl Encodable for String {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.encoder().encode_value(self.as_bytes());
	}
}

impl Decodable for String {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		rlp.decoder().decode_value(|bytes| {
			match str::from_utf8(bytes) {
				Ok(s) => Ok(s.to_owned()),
				// consider better error type here
				Err(_err) => Err(DecoderError::RlpExpectedToBeData),
			}
		})
	}
}
