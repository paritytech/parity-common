// Copyright 2021 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![no_main]

use libfuzzer_sys::fuzz_target;
use uint::*;

construct_uint! {
	pub struct U256(4);
}

fn isqrt(mut me: U256) -> U256 {
	let one = U256::one();
	if me <= one {
		return me;
	}
	// the implementation is based on:
	// https://en.wikipedia.org/wiki/Methods_of_computing_square_roots#Binary_numeral_system_(base_2)

	// "bit" starts at the highest power of four <= self.
	let max_shift = 4 * 64 as u32 - 1;
	let shift: u32 = (max_shift - me.leading_zeros()) & !1;
	let mut bit = one << shift;
	let mut result = U256::zero();
	while !bit.is_zero() {
		let x = result + bit;
		result >>= 1;
		if me >= x {
			me -= x;
			result += bit;
		}
		bit >>= 2;
	}
	result
}

fuzz_target!(|data: &[u8]| {
	if data.len() == 32 {
		let x = U256::from_little_endian(data);
		let expected = isqrt(x);
		let got = x.integer_sqrt();
		assert_eq!(got, expected);
	}
});
