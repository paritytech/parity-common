// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature = "codec")]
use impl_codec::impl_uint_codec;
#[cfg(feature = "rlp")]
use impl_rlp::impl_uint_rlp;
#[cfg(feature = "serialize")]
use impl_serde::impl_uint_serde;
use uint_crate::*;

pub use uint_crate::{FromDecStrErr, FromStrRadixErr, FromStrRadixErrKind};

construct_uint! {
	/// Unsigned 64-bit integer.
	pub struct U64(1);
}
#[cfg(feature = "rlp")]
impl_uint_rlp!(U64, 1);
#[cfg(feature = "serialize")]
impl_uint_serde!(U64, 1);
#[cfg(feature = "codec")]
impl_uint_codec!(U64, 1);

pub use primitive_types::{U128, U256, U512};

#[cfg(test)]
mod tests {
	use super::{U256, U512};
	use serde_json as ser;
	use std::u64::MAX;

	macro_rules! test_serialize {
		($name: ident, $test_name: ident) => {
			#[test]
			fn $test_name() {
				let tests = vec![
					($name::from(0), "0x0"),
					($name::from(1), "0x1"),
					($name::from(2), "0x2"),
					($name::from(10), "0xa"),
					($name::from(15), "0xf"),
					($name::from(15), "0xf"),
					($name::from(16), "0x10"),
					($name::from(1_000), "0x3e8"),
					($name::from(100_000), "0x186a0"),
					($name::from(u64::max_value()), "0xffffffffffffffff"),
					($name::from(u64::max_value()) + 1, "0x10000000000000000"),
				];

				for (number, expected) in tests {
					assert_eq!(format!("{:?}", expected), ser::to_string_pretty(&number).unwrap());
					assert_eq!(number, ser::from_str(&format!("{:?}", expected)).unwrap());
				}

				// Invalid examples
				assert!(ser::from_str::<$name>("\"0x\"").unwrap_err().is_data());
				assert!(ser::from_str::<$name>("\"0xg\"").unwrap_err().is_data());
				assert!(ser::from_str::<$name>("\"\"").unwrap_err().is_data());
				assert!(ser::from_str::<$name>("\"10\"").unwrap_err().is_data());
				assert!(ser::from_str::<$name>("\"0\"").unwrap_err().is_data());
			}
		};
	}

	test_serialize!(U256, test_u256);
	test_serialize!(U512, test_u512);

	#[test]
	fn test_serialize_large_values() {
		assert_eq!(
			ser::to_string_pretty(&!U256::zero()).unwrap(),
			"\"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\""
		);
		assert!(ser::from_str::<U256>("\"0x1ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\"")
			.unwrap_err()
			.is_data());
	}

	#[test]
	fn fixed_arrays_roundtrip() {
		let raw: U256 = "7094875209347850239487502394881".into();
		let array: [u8; 32] = raw.into();
		let new_raw = array.into();

		assert_eq!(raw, new_raw);
	}

	#[test]
	fn u256_multi_full_mul() {
		let result = U256([0, 0, 0, 0]).full_mul(U256([0, 0, 0, 0]));
		assert_eq!(U512([0, 0, 0, 0, 0, 0, 0, 0]), result);

		let result = U256([1, 0, 0, 0]).full_mul(U256([1, 0, 0, 0]));
		assert_eq!(U512([1, 0, 0, 0, 0, 0, 0, 0]), result);

		let result = U256([5, 0, 0, 0]).full_mul(U256([5, 0, 0, 0]));
		assert_eq!(U512([25, 0, 0, 0, 0, 0, 0, 0]), result);

		let result = U256([0, 5, 0, 0]).full_mul(U256([0, 5, 0, 0]));
		assert_eq!(U512([0, 0, 25, 0, 0, 0, 0, 0]), result);

		let result = U256([0, 0, 0, 4]).full_mul(U256([4, 0, 0, 0]));
		assert_eq!(U512([0, 0, 0, 16, 0, 0, 0, 0]), result);

		let result = U256([0, 0, 0, 5]).full_mul(U256([2, 0, 0, 0]));
		assert_eq!(U512([0, 0, 0, 10, 0, 0, 0, 0]), result);

		let result = U256([0, 0, 2, 0]).full_mul(U256([0, 5, 0, 0]));
		assert_eq!(U512([0, 0, 0, 10, 0, 0, 0, 0]), result);

		let result = U256([0, 3, 0, 0]).full_mul(U256([0, 0, 3, 0]));
		assert_eq!(U512([0, 0, 0, 9, 0, 0, 0, 0]), result);

		let result = U256([0, 0, 8, 0]).full_mul(U256([0, 0, 6, 0]));
		assert_eq!(U512([0, 0, 0, 0, 48, 0, 0, 0]), result);

		let result = U256([9, 0, 0, 0]).full_mul(U256([0, 3, 0, 0]));
		assert_eq!(U512([0, 27, 0, 0, 0, 0, 0, 0]), result);

		let result = U256([MAX, 0, 0, 0]).full_mul(U256([MAX, 0, 0, 0]));
		assert_eq!(U512([1, MAX - 1, 0, 0, 0, 0, 0, 0]), result);

		let result = U256([0, MAX, 0, 0]).full_mul(U256([MAX, 0, 0, 0]));
		assert_eq!(U512([0, 1, MAX - 1, 0, 0, 0, 0, 0]), result);

		let result = U256([MAX, MAX, 0, 0]).full_mul(U256([MAX, 0, 0, 0]));
		assert_eq!(U512([1, MAX, MAX - 1, 0, 0, 0, 0, 0]), result);

		let result = U256([MAX, 0, 0, 0]).full_mul(U256([MAX, MAX, 0, 0]));
		assert_eq!(U512([1, MAX, MAX - 1, 0, 0, 0, 0, 0]), result);

		let result = U256([MAX, MAX, 0, 0]).full_mul(U256([MAX, MAX, 0, 0]));
		assert_eq!(U512([1, 0, MAX - 1, MAX, 0, 0, 0, 0]), result);

		let result = U256([MAX, 0, 0, 0]).full_mul(U256([MAX, MAX, MAX, 0]));
		assert_eq!(U512([1, MAX, MAX, MAX - 1, 0, 0, 0, 0]), result);

		let result = U256([MAX, MAX, MAX, 0]).full_mul(U256([MAX, 0, 0, 0]));
		assert_eq!(U512([1, MAX, MAX, MAX - 1, 0, 0, 0, 0]), result);

		let result = U256([MAX, 0, 0, 0]).full_mul(U256([MAX, MAX, MAX, MAX]));
		assert_eq!(U512([1, MAX, MAX, MAX, MAX - 1, 0, 0, 0]), result);

		let result = U256([MAX, MAX, MAX, MAX]).full_mul(U256([MAX, 0, 0, 0]));
		assert_eq!(U512([1, MAX, MAX, MAX, MAX - 1, 0, 0, 0]), result);

		let result = U256([MAX, MAX, MAX, 0]).full_mul(U256([MAX, MAX, 0, 0]));
		assert_eq!(U512([1, 0, MAX, MAX - 1, MAX, 0, 0, 0]), result);

		let result = U256([MAX, MAX, 0, 0]).full_mul(U256([MAX, MAX, MAX, 0]));
		assert_eq!(U512([1, 0, MAX, MAX - 1, MAX, 0, 0, 0]), result);

		let result = U256([MAX, MAX, MAX, MAX]).full_mul(U256([MAX, MAX, 0, 0]));
		assert_eq!(U512([1, 0, MAX, MAX, MAX - 1, MAX, 0, 0]), result);

		let result = U256([MAX, MAX, 0, 0]).full_mul(U256([MAX, MAX, MAX, MAX]));
		assert_eq!(U512([1, 0, MAX, MAX, MAX - 1, MAX, 0, 0]), result);

		let result = U256([MAX, MAX, MAX, 0]).full_mul(U256([MAX, MAX, MAX, 0]));
		assert_eq!(U512([1, 0, 0, MAX - 1, MAX, MAX, 0, 0]), result);

		let result = U256([MAX, MAX, MAX, 0]).full_mul(U256([MAX, MAX, MAX, MAX]));
		assert_eq!(U512([1, 0, 0, MAX, MAX - 1, MAX, MAX, 0]), result);

		let result = U256([MAX, MAX, MAX, MAX]).full_mul(U256([MAX, MAX, MAX, 0]));
		assert_eq!(U512([1, 0, 0, MAX, MAX - 1, MAX, MAX, 0]), result);

		let result = U256([MAX, MAX, MAX, MAX]).full_mul(U256([MAX, MAX, MAX, MAX]));
		assert_eq!(U512([1, 0, 0, 0, MAX - 1, MAX, MAX, MAX]), result);

		let result = U256([0, 0, 0, MAX]).full_mul(U256([0, 0, 0, MAX]));
		assert_eq!(U512([0, 0, 0, 0, 0, 0, 1, MAX - 1]), result);

		let result = U256([1, 0, 0, 0]).full_mul(U256([0, 0, 0, MAX]));
		assert_eq!(U512([0, 0, 0, MAX, 0, 0, 0, 0]), result);

		let result = U256([1, 2, 3, 4]).full_mul(U256([5, 0, 0, 0]));
		assert_eq!(U512([5, 10, 15, 20, 0, 0, 0, 0]), result);

		let result = U256([1, 2, 3, 4]).full_mul(U256([0, 6, 0, 0]));
		assert_eq!(U512([0, 6, 12, 18, 24, 0, 0, 0]), result);

		let result = U256([1, 2, 3, 4]).full_mul(U256([0, 0, 7, 0]));
		assert_eq!(U512([0, 0, 7, 14, 21, 28, 0, 0]), result);

		let result = U256([1, 2, 3, 4]).full_mul(U256([0, 0, 0, 8]));
		assert_eq!(U512([0, 0, 0, 8, 16, 24, 32, 0]), result);

		let result = U256([1, 2, 3, 4]).full_mul(U256([5, 6, 7, 8]));
		assert_eq!(U512([5, 16, 34, 60, 61, 52, 32, 0]), result);
	}
}
