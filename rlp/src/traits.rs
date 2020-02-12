// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Common RLP traits
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::error::DecoderError;
use crate::rlpin::Rlp;
use crate::stream::RlpStream;

/// RLP decodable trait
pub trait Decodable: Sized {
	/// Decode a value from RLP bytes
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError>;
}

/// Structure encodable to RLP
pub trait Encodable {
	/// Append a value to the stream
	fn rlp_append(&self, s: &mut RlpStream);

	/// Get rlp-encoded bytes for this instance
	fn rlp_bytes(&self) -> Vec<u8> {
		let mut s = RlpStream::new();
		self.rlp_append(&mut s);
		s.drain()
	}
}
