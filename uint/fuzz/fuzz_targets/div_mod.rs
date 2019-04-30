#![no_main]

use libfuzzer_sys::fuzz_target;
use uint::*;


construct_uint! {
	pub struct U512(8);
}

fuzz_target!(|data: &[u8]| {
    if data.len() == 128 {
		let x = U512::from_little_endian(&data[..64]);
		let y = U512::from_little_endian(&data[64..128]);
		if !y.is_zero() {
			let divmod = (x / y, x % y);
			assert_eq!(divmod, x.div_mod(y));
		}
    }
});
