// Copyright 2015-2017 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[doc(hidden)]
pub extern crate rlp;

#[doc(hidden)]
pub extern crate core as core_;

#[macro_export]
macro_rules! impl_uint_rlp {
	($name: ident, $size: expr) => {
		impl $crate::rlp::Encodable for $name {
			fn rlp_append(&self, s: &mut $crate::rlp::RlpStream) {
				let leading_empty_bytes = $size - (self.bits() + 7) / 8;
				let mut buffer = [0u8; $size];
				self.to_big_endian(&mut buffer);
				s.encoder().encode_value(&buffer[leading_empty_bytes..]);
			}
		}

		impl $crate::rlp::Decodable for $name {
			fn decode(rlp: &$crate::rlp::Rlp) -> Result<Self, $crate::rlp::DecoderError> {
				rlp.decoder().decode_value(|bytes| {
					if !bytes.is_empty() && bytes[0] == 0 {
						Err($crate::rlp::DecoderError::RlpInvalidIndirection)
					} else if bytes.len() <= $size {
						Ok($name::from(bytes))
					} else {
						Err($crate::rlp::DecoderError::RlpIsTooBig)
					}
				})
			}
		}
	}
}

#[macro_export]
macro_rules! impl_fixed_hash_rlp {
	($name: ident, $size: expr) => {
		impl $crate::rlp::Encodable for $name {
			fn rlp_append(&self, s: &mut $crate::rlp::RlpStream) {
				s.encoder().encode_value(self.as_ref());
			}
		}

		impl $crate::rlp::Decodable for $name {
			fn decode(rlp: &$crate::rlp::Rlp) -> Result<Self, $crate::rlp::DecoderError> {
				rlp.decoder().decode_value(|bytes| match bytes.len().cmp(&$size) {
					$crate::core_::cmp::Ordering::Less => Err($crate::rlp::DecoderError::RlpIsTooShort),
					$crate::core_::cmp::Ordering::Greater => Err($crate::rlp::DecoderError::RlpIsTooBig),
					$crate::core_::cmp::Ordering::Equal => {
						let mut t = [0u8; $size];
						t.copy_from_slice(bytes);
						Ok($name(t))
					}
				})
			}
		}
	}
}
