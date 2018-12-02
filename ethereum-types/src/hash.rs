use crate::{U64, U128, U256, U512};

pub trait BigEndianHash {
	type Uint;

	fn from_uint(val: &Self::Uint) -> Self;
	fn into_uint(&self) -> Self::Uint;
}

construct_fixed_hash!{ pub struct H32(4); }
impl_fixed_hash_rlp!(H32, 4);
#[cfg(feature = "serialize")] impl_fixed_hash_serde!(H32, 4);

construct_fixed_hash!{ pub struct H64(8); }
impl_fixed_hash_rlp!(H64, 8);
#[cfg(feature = "serialize")] impl_fixed_hash_serde!(H64, 8);

construct_fixed_hash!{ pub struct H128(16); }
impl_fixed_hash_rlp!(H128, 16);
#[cfg(feature = "serialize")] impl_fixed_hash_serde!(H128, 16);

pub use primitive_types::H160;
pub use primitive_types::H256;

construct_fixed_hash!{ pub struct H264(33); }
impl_fixed_hash_rlp!(H264, 33);
#[cfg(feature = "serialize")] impl_fixed_hash_serde!(H264, 33);

pub use primitive_types::H512;

construct_fixed_hash!{ pub struct H520(65); }
impl_fixed_hash_rlp!(H520, 65);
#[cfg(feature = "serialize")] impl_fixed_hash_serde!(H520, 65);

macro_rules! impl_uint_conversions {
	($hash: ident, $uint: ident) => {
		impl BigEndianHash for $hash {
			type Uint = $uint;

			fn from_uint(value: &$uint) -> Self {
				let mut ret = $hash::zero();
				value.to_big_endian(ret.as_bytes_mut());
				ret
			}

			fn into_uint(&self) -> $uint {
				$uint::from(self.as_ref() as &[u8])
			}
		}
	}
}

impl_uint_conversions!(H64, U64);
impl_uint_conversions!(H128, U128);
impl_uint_conversions!(H256, U256);
impl_uint_conversions!(H512, U512);

#[cfg(test)]
mod tests {
	use super::{H160, H256};
	use serde_json as ser;

	#[test]
	fn test_serialize_h160() {
		let tests = vec![
			(H160::from(0), "0x0000000000000000000000000000000000000000"),
			(H160::from(2), "0x0000000000000000000000000000000000000002"),
			(H160::from(15), "0x000000000000000000000000000000000000000f"),
			(H160::from(16), "0x0000000000000000000000000000000000000010"),
			(H160::from(1_000), "0x00000000000000000000000000000000000003e8"),
			(H160::from(100_000), "0x00000000000000000000000000000000000186a0"),
			(H160::from(u64::max_value()), "0x000000000000000000000000ffffffffffffffff"),
		];

		for (number, expected) in tests {
			assert_eq!(format!("{:?}", expected), ser::to_string_pretty(&number).unwrap());
			assert_eq!(number, ser::from_str(&format!("{:?}", expected)).unwrap());
		}
	}

	#[test]
	fn test_serialize_h256() {
		let tests = vec![
			(H256::from(0), "0x0000000000000000000000000000000000000000000000000000000000000000"),
			(H256::from(2), "0x0000000000000000000000000000000000000000000000000000000000000002"),
			(H256::from(15), "0x000000000000000000000000000000000000000000000000000000000000000f"),
			(H256::from(16), "0x0000000000000000000000000000000000000000000000000000000000000010"),
			(H256::from(1_000), "0x00000000000000000000000000000000000000000000000000000000000003e8"),
			(H256::from(100_000), "0x00000000000000000000000000000000000000000000000000000000000186a0"),
			(H256::from(u64::max_value()), "0x000000000000000000000000000000000000000000000000ffffffffffffffff"),
		];

		for (number, expected) in tests {
			assert_eq!(format!("{:?}", expected), ser::to_string_pretty(&number).unwrap());
			assert_eq!(number, ser::from_str(&format!("{:?}", expected)).unwrap());
		}
	}

	#[test]
	fn test_serialize_invalid() {
		assert!(ser::from_str::<H256>("\"0x000000000000000000000000000000000000000000000000000000000000000\"").unwrap_err().is_data());
		assert!(ser::from_str::<H256>("\"0x000000000000000000000000000000000000000000000000000000000000000g\"").unwrap_err().is_data());
		assert!(ser::from_str::<H256>("\"0x00000000000000000000000000000000000000000000000000000000000000000\"").unwrap_err().is_data());
		assert!(ser::from_str::<H256>("\"\"").unwrap_err().is_data());
		assert!(ser::from_str::<H256>("\"0\"").unwrap_err().is_data());
		assert!(ser::from_str::<H256>("\"10\"").unwrap_err().is_data());
	}
}
