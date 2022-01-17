// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Recursive Length Prefix serialization crate.
//!
//! Allows encoding, decoding, and view onto rlp-slice
//!
//! # What should you use when?
//!
//! ### Use `encode` function when:
//! * You want to encode something inline.
//! * You do not work on big set of data.
//! * You want to encode whole data structure at once.
//!
//! ### Use `decode` function when:
//! * You want to decode something inline.
//! * You do not work on big set of data.
//! * You want to decode whole rlp at once.
//!
//! ### Use `RlpStream` when:
//! * You want to encode something in portions.
//! * You encode a big set of data.
//!
//! ### Use `Rlp` when:
//! * You need to handle data corruption errors.
//! * You are working on input data.
//! * You want to get view onto rlp-slice.
//! * You don't want to decode whole rlp at once.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod error;
mod impls;
mod rlpin;
mod stream;
mod traits;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use bytes::BytesMut;
use core::borrow::Borrow;

#[cfg(feature = "derive")]
pub use rlp_derive::{RlpDecodable, RlpDecodableWrapper, RlpEncodable, RlpEncodableWrapper};

pub use self::{
	error::DecoderError,
	rlpin::{PayloadInfo, Prototype, Rlp, RlpIterator},
	stream::RlpStream,
	traits::{Decodable, Encodable},
};

/// The RLP encoded empty data (used to mean "null value").
pub const NULL_RLP: [u8; 1] = [0x80; 1];
/// The RLP encoded empty list.
pub const EMPTY_LIST_RLP: [u8; 1] = [0xC0; 1];

/// Shortcut function to decode trusted rlp
///
/// ```
/// let data = vec![0x83, b'c', b'a', b't'];
/// let animal: String = rlp::decode(&data).expect("could not decode");
/// assert_eq!(animal, "cat".to_owned());
/// ```
pub fn decode<T>(bytes: &[u8]) -> Result<T, DecoderError>
where
	T: Decodable,
{
	let rlp = Rlp::new(bytes);
	rlp.as_val()
}

pub fn decode_list<T>(bytes: &[u8]) -> Vec<T>
where
	T: Decodable,
{
	let rlp = Rlp::new(bytes);
	rlp.as_list().expect("trusted rlp should be valid")
}

/// Shortcut function to encode structure into rlp.
///
/// ```
/// let animal = "cat";
/// let out = rlp::encode(&animal);
/// assert_eq!(out, vec![0x83, b'c', b'a', b't']);
/// ```
pub fn encode<E>(object: &E) -> BytesMut
where
	E: Encodable,
{
	let mut stream = RlpStream::new();
	stream.append(object);
	stream.out()
}

pub fn encode_list<E, K>(object: &[K]) -> BytesMut
where
	E: Encodable,
	K: Borrow<E>,
{
	let mut stream = RlpStream::new();
	stream.append_list(object);
	stream.out()
}
