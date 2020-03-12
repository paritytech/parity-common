// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::{KEY_LENGTH, KEY_LENGTH_AES};
use crate::error::ScryptError;
use scrypt::{scrypt, ScryptParams};

#[cfg(test)]
use std::io::Error;

pub fn derive_key(pass: &[u8], salt: &[u8], n: u32, p: u32, r: u32) -> Result<(Vec<u8>, Vec<u8>), ScryptError> {
	// sanity checks
	let log_n = (32 - n.leading_zeros() - 1) as u8;
	if log_n as u32 >= r * 16 {
		return Err(ScryptError::InvalidN);
	}

	if p as u64 > ((u32::max_value() as u64 - 1) * 32) / (128 * (r as u64)) {
		return Err(ScryptError::InvalidP);
	}

	let mut derived_key = vec![0u8; KEY_LENGTH];
	let scrypt_params = ScryptParams::new(log_n, r, p)?;
	scrypt(pass, salt, &scrypt_params, &mut derived_key)?;
	let derived_right_bits = &derived_key[0..KEY_LENGTH_AES];
	let derived_left_bits = &derived_key[KEY_LENGTH_AES..KEY_LENGTH];
	Ok((derived_right_bits.to_vec(), derived_left_bits.to_vec()))
}

// test is build from previous crypto lib behaviour, values may be incorrect
// if previous crypto lib got a bug.
#[test]
pub fn test_derive() -> Result<(), Error> {
	let pass = [109, 121, 112, 97, 115, 115, 10];
	let salt = [
		109, 121, 115, 97, 108, 116, 115, 104, 111, 117, 108, 100, 102, 105, 108, 108, 115, 111, 109, 109, 101, 98,
		121, 116, 101, 108, 101, 110, 103, 116, 104, 10,
	];
	let r1 = [93, 134, 79, 68, 223, 27, 44, 174, 236, 184, 179, 203, 74, 139, 73, 66];
	let r2 = [2, 24, 239, 131, 172, 164, 18, 171, 132, 207, 22, 217, 150, 20, 203, 37];
	let l1 = [6, 90, 119, 45, 67, 2, 99, 151, 81, 88, 166, 210, 244, 19, 123, 208];
	let l2 = [253, 123, 132, 12, 188, 89, 196, 2, 107, 224, 239, 231, 135, 177, 125, 62];

	let (l, r) = derive_key(&pass[..], &salt, 262, 1, 8).unwrap();
	assert!(l == r1);
	assert!(r == l1);
	let (l, r) = derive_key(&pass[..], &salt, 144, 4, 4).unwrap();
	assert!(l == r2);
	assert!(r == l2);
	Ok(())
}
