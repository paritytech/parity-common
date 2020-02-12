// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ethereum_types::{H160, H256, U256, U512};
use serde_json as ser;

macro_rules! test {
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
				($name::from(u64::max_value()) + $name::from(1u64), "0x10000000000000000"),
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

test!(U256, test_u256);
test!(U512, test_u512);

#[test]
fn test_large_values() {
	assert_eq!(
		ser::to_string_pretty(&!U256::zero()).unwrap(),
		"\"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\""
	);
	assert!(ser::from_str::<U256>("\"0x1ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\"")
		.unwrap_err()
		.is_data());
}

#[test]
fn test_h160() {
	let tests = vec![
		(H160::from_low_u64_be(0), "0x0000000000000000000000000000000000000000"),
		(H160::from_low_u64_be(2), "0x0000000000000000000000000000000000000002"),
		(H160::from_low_u64_be(15), "0x000000000000000000000000000000000000000f"),
		(H160::from_low_u64_be(16), "0x0000000000000000000000000000000000000010"),
		(H160::from_low_u64_be(1_000), "0x00000000000000000000000000000000000003e8"),
		(H160::from_low_u64_be(100_000), "0x00000000000000000000000000000000000186a0"),
		(H160::from_low_u64_be(u64::max_value()), "0x000000000000000000000000ffffffffffffffff"),
	];

	for (number, expected) in tests {
		assert_eq!(format!("{:?}", expected), ser::to_string_pretty(&number).unwrap());
		assert_eq!(number, ser::from_str(&format!("{:?}", expected)).unwrap());
	}
}

#[test]
fn test_h256() {
	let tests = vec![
		(H256::from_low_u64_be(0), "0x0000000000000000000000000000000000000000000000000000000000000000"),
		(H256::from_low_u64_be(2), "0x0000000000000000000000000000000000000000000000000000000000000002"),
		(H256::from_low_u64_be(15), "0x000000000000000000000000000000000000000000000000000000000000000f"),
		(H256::from_low_u64_be(16), "0x0000000000000000000000000000000000000000000000000000000000000010"),
		(H256::from_low_u64_be(1_000), "0x00000000000000000000000000000000000000000000000000000000000003e8"),
		(H256::from_low_u64_be(100_000), "0x00000000000000000000000000000000000000000000000000000000000186a0"),
		(H256::from_low_u64_be(u64::max_value()), "0x000000000000000000000000000000000000000000000000ffffffffffffffff"),
	];

	for (number, expected) in tests {
		assert_eq!(format!("{:?}", expected), ser::to_string_pretty(&number).unwrap());
		assert_eq!(number, ser::from_str(&format!("{:?}", expected)).unwrap());
	}
}

#[test]
fn test_invalid() {
	assert!(ser::from_str::<H256>("\"0x000000000000000000000000000000000000000000000000000000000000000\"")
		.unwrap_err()
		.is_data());
	assert!(ser::from_str::<H256>("\"0x000000000000000000000000000000000000000000000000000000000000000g\"")
		.unwrap_err()
		.is_data());
	assert!(ser::from_str::<H256>("\"0x00000000000000000000000000000000000000000000000000000000000000000\"")
		.unwrap_err()
		.is_data());
	assert!(ser::from_str::<H256>("\"\"").unwrap_err().is_data());
	assert!(ser::from_str::<H256>("\"0\"").unwrap_err().is_data());
	assert!(ser::from_str::<H256>("\"10\"").unwrap_err().is_data());
}

#[test]
fn test_invalid_char() {
	const INVALID_STR: &str = "\"0x000000000000000000000000000000000000000000000000000000000000000g\"";
	const EXPECTED_MSG: &str = "invalid hex character: g, at 65 at line 1 column 68";
	assert_eq!(ser::from_str::<H256>(INVALID_STR).unwrap_err().to_string(), EXPECTED_MSG);
}
