// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![no_main]

use libfuzzer_sys::fuzz_target;

fn split(a: u64) -> (u64, u64) {
	(a >> 32, a & 0xFFFF_FFFF)
}

fn div_mod_word_u128(hi: u64, lo: u64, d: u64) -> (u64, u64) {
	let x = (u128::from(hi) << 64) + u128::from(lo);
	let d = u128::from(d);
	((x / d) as u64, (x % d) as u64)
}

fn div_mod_word(hi: u64, lo: u64, y: u64) -> (u64, u64) {
	debug_assert!(hi < y);
	const TWO32: u64 = 1 << 32;
	let s = y.leading_zeros();
	let y = y << s;
	let (yn1, yn0) = split(y);
	let un32 = (hi << s) | lo.checked_shr(64 - s).unwrap_or(0);
	let un10 = lo << s;
	let (un1, un0) = split(un10);
	let mut q1 = un32 / yn1;
	let mut rhat = un32 - q1 * yn1;

	while q1 >= TWO32 || q1 * yn0 > TWO32 * rhat + un1 {
		q1 -= 1;
		rhat += yn1;
		if rhat >= TWO32 {
			break;
		}
	}

	let un21 = un32.wrapping_mul(TWO32).wrapping_add(un1).wrapping_sub(q1.wrapping_mul(y));
	let mut q0 = un21 / yn1;
	rhat = un21.wrapping_sub(q0.wrapping_mul(yn1));

	while q0 >= TWO32 || q0 * yn0 > TWO32 * rhat + un0 {
		q0 -= 1;
		rhat += yn1;
		if rhat >= TWO32 {
			break;
		}
	}

	let rem = un21.wrapping_mul(TWO32).wrapping_add(un0).wrapping_sub(y.wrapping_mul(q0));
	(q1 * TWO32 + q0, rem >> s)
}

fuzz_target!(|data: &[u8]| {
    if data.len() == 24 {
		let mut buf = [0u8; 8];
		buf.copy_from_slice(&data[..8]);
		let x = u64::from_ne_bytes(buf);
		buf.copy_from_slice(&data[8..16]);
		let y = u64::from_ne_bytes(buf);
		buf.copy_from_slice(&data[16..24]);
		let z = u64::from_ne_bytes(buf);
		if x < z {
			assert_eq!(div_mod_word(x, y, z), div_mod_word_u128(x, y, z));
		}
    }
});
