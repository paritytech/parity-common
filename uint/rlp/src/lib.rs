#[doc(hidden)]
pub extern crate rlp;

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
