// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Testing to and from f64 lossy for U256 primitive type.

use primitive_types::U256;

#[test]
#[allow(clippy::float_cmp)]
fn convert_u256_to_f64() {
	assert_eq!(U256::from(0).to_f64_lossy(), 0.0);
	assert_eq!(U256::from(42).to_f64_lossy(), 42.0);
	assert_eq!(U256::from(1_000_000_000_000_000_000u128).to_f64_lossy(), 1_000_000_000_000_000_000.0,);
}

#[test]
#[allow(clippy::excessive_precision, clippy::float_cmp, clippy::unreadable_literal)]
#[cfg(feature = "std")]
fn convert_u256_to_f64_precision_loss() {
	assert_eq!(U256::from(u64::max_value()).to_f64_lossy(), u64::max_value() as f64,);
	assert_eq!(
		U256::MAX.to_f64_lossy(),
		115792089237316195423570985008687907853269984665640564039457584007913129639935.0,
	);
	assert_eq!(
		U256::MAX.to_f64_lossy(),
		115792089237316200000000000000000000000000000000000000000000000000000000000000.0,
	);
}

#[test]
fn convert_f64_to_u256() {
	assert_eq!(U256::from_f64_lossy(0.0), 0.into());
	assert_eq!(U256::from_f64_lossy(13.37), 13.into());
	assert_eq!(U256::from_f64_lossy(42.0), 42.into());
	assert_eq!(U256::from_f64_lossy(999.999), 999.into());
	assert_eq!(U256::from_f64_lossy(1_000_000_000_000_000_000.0), 1_000_000_000_000_000_000u128.into(),);
}

#[test]
fn convert_f64_to_u256_large() {
	let value = U256::from(1) << U256::from(255);
	assert_eq!(U256::from_f64_lossy(format!("{}", value).parse::<f64>().expect("unexpected error parsing f64")), value);
}

#[test]
#[allow(clippy::unreadable_literal)]
fn convert_f64_to_u256_overflow() {
	assert_eq!(
		U256::from_f64_lossy(115792089237316200000000000000000000000000000000000000000000000000000000000000.0),
		U256::MAX,
	);
	assert_eq!(
		U256::from_f64_lossy(999999999999999999999999999999999999999999999999999999999999999999999999999999.0),
		U256::MAX,
	);
}

#[test]
fn convert_f64_to_u256_non_normal() {
	assert_eq!(U256::from_f64_lossy(f64::EPSILON), 0.into());
	assert_eq!(U256::from_f64_lossy(f64::from_bits(0)), 0.into());
	assert_eq!(U256::from_f64_lossy(f64::NAN), 0.into());
	assert_eq!(U256::from_f64_lossy(f64::NEG_INFINITY), 0.into());
	assert_eq!(U256::from_f64_lossy(f64::INFINITY), U256::MAX);
}

#[test]
fn f64_to_u256_truncation() {
	assert_eq!(U256::from_f64_lossy(10.5), 10.into());
}
