// Copyright 2021 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use impl_num_traits::integer_sqrt::IntegerSquareRoot;
use primitive_types::U256;

#[test]
fn u256_isqrt() {
	let x = U256::MAX;
	let s = x.integer_sqrt_checked().unwrap();
	assert_eq!(x.integer_sqrt(), s);
}
