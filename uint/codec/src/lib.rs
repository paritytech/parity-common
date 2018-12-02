#[doc(hidden)]
pub extern crate parity_codec as codec;

#[macro_export]
macro_rules! impl_uint_codec {
	($name: ident, $len: expr) => {
		impl $crate::codec::Encode for $name {
			fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
				let mut bytes = [0u8; $len * 8];
				self.to_little_endian(&mut bytes);
				bytes.using_encoded(f)
			}
		}

		impl $crate::codec::Decode for $name {
			fn decode<I: $crate::codec::Input>(input: &mut I) -> Option<Self> {
				<[u8; $len * 8] as $crate::codec::Decode>::decode(input)
					.map(|b| $name::from_little_endian(&b))
			}
		}
	}
}
