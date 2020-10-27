// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Tests for scale-info feature of primitive-types.

use primitive_types::{H256, U256};
use scale_info::{build::Fields, Path, Type, TypeInfo};

#[test]
fn u256_scale_info() {
	let r#type =
		Type::builder().path(Path::new("U256", "primitive_types")).composite(Fields::unnamed().field_of::<[u64; 4]>());

	assert_eq!(U256::type_info(), r#type.into());
}

#[test]
fn h256_scale_info() {
	let r#type =
		Type::builder().path(Path::new("H256", "primitive_types")).composite(Fields::unnamed().field_of::<[u8; 32]>());

	assert_eq!(H256::type_info(), r#type.into());
}

#[test]
#[allow(clippy::float_cmp)]
fn convert_u256_to_f64() {
	assert_eq!(f64::from(0.into()), 0.0);
	assert_eq!(f64::from(42.into()), 42.0);
	assert_eq!(f64::from(1_000_000_000_000_000_000u128.into()), 1_000_000_000_000_000_000.0,);
}

#[test]
#[allow(clippy::excessive_precision, clippy::float_cmp, clippy::unreadable_literal)]
fn convert_u256_to_f64_precision_loss() {
	assert_eq!(f64::from(u64::max_value().into()), u64::max_value() as f64,);
	assert_eq!(f64::from(U256::MAX), 115792089237316195423570985008687907853269984665640564039457584007913129639935.0,);
	assert_eq!(f64::from(U256::MAX), 115792089237316200000000000000000000000000000000000000000000000000000000000000.0,);
}

#[test]
fn convert_f64_to_u256() {
	assert_eq!(U256::from(0.0), 0.into());
	assert_eq!(U256::from(13.37), 13.into());
	assert_eq!(U256::from(42.0), 42.into());
	assert_eq!(U256::from(999.999), 999.into());
	assert_eq!(U256::from(1_000_000_000_000_000_000.0), 1_000_000_000_000_000_000u128.into(),);
}

#[test]
fn convert_f64_to_u256_large() {
	let value = U256::from(1) << U256::from(255);
	assert_eq!(U256::from(format!("{}", value).parse::<f64>().expect("unexpected error parsing f64")), value,);
}

#[test]
#[allow(clippy::unreadable_literal)]
fn convert_f64_to_u256_overflow() {
	assert_eq!(U256::from(115792089237316200000000000000000000000000000000000000000000000000000000000000.0), U256::MAX,);
	assert_eq!(U256::from(999999999999999999999999999999999999999999999999999999999999999999999999999999.0), U256::MAX,);
}

#[test]
fn convert_f64_to_u256_non_normal() {
	assert_eq!(U256::from(f64::EPSILON), 0.into());
	assert_eq!(U256::from(f64::from_bits(0)), 0.into());
	assert_eq!(U256::from(f64::NAN), 0.into());
	assert_eq!(U256::from(f64::NEG_INFINITY), 0.into());
	assert_eq!(U256::from(f64::INFINITY), U256::MAX);
}
