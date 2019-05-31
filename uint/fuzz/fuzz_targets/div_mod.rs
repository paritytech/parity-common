#![no_main]

use libfuzzer_sys::fuzz_target;
use uint::*;
use rug::{Integer, integer::Order};


construct_uint! {
	pub struct U512(8);
}

fn from_gmp(x: Integer) -> U512 {
	let digits = x.to_digits(Order::LsfLe);
	U512::from_little_endian(&digits)
}

fuzz_target!(|data: &[u8]| {
    if data.len() == 128 {
		let x = U512::from_little_endian(&data[..64]);
		let y = U512::from_little_endian(&data[64..]);
		let x_gmp = Integer::from_digits(&data[..64], Order::LsfLe);
		let y_gmp = Integer::from_digits(&data[64..], Order::LsfLe);
		if !y.is_zero() {
			let (a, b) = x_gmp.div_rem(y_gmp);
			assert_eq!((from_gmp(a), from_gmp(b)), x.div_mod(y));
		}
    }
});
