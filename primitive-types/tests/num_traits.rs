// Copyright 2021 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use impl_num_traits::integer_sqrt::IntegerSquareRoot;
use num_traits::ops::checked::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use primitive_types::U256;

#[test]
fn u256_isqrt() {
	let x = U256::MAX;
	let s = x.integer_sqrt_checked().unwrap();
	assert_eq!(x.integer_sqrt(), s);
}

#[test]
fn u256_checked_traits_supported() {
	const ZERO: &U256 = &U256::zero();
	const ONE: &U256 = &U256::one();
	const MAX: &U256 = &U256::MAX;

	assert_eq!(<U256 as CheckedAdd>::checked_add(MAX, ONE), None);
	assert_eq!(<U256 as CheckedAdd>::checked_add(ZERO, ONE), Some(*ONE));

	assert_eq!(<U256 as CheckedSub>::checked_sub(ZERO, ONE), None);
	assert_eq!(<U256 as CheckedSub>::checked_sub(ONE, ZERO), Some(*ONE));

	assert_eq!(<U256 as CheckedDiv>::checked_div(MAX, ZERO), None);
	assert_eq!(<U256 as CheckedDiv>::checked_div(MAX, ONE), Some(*MAX));

	assert_eq!(<U256 as CheckedMul>::checked_mul(MAX, MAX), None);
	assert_eq!(<U256 as CheckedMul>::checked_mul(MAX, ZERO), Some(*ZERO));
}
