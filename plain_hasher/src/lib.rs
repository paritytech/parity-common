// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(not(feature = "std"), no_std)]

use core::hash::Hasher;

use crunchy::unroll;

/// Hasher that just takes 8 bytes of the provided value.
/// May only be used for keys which are 32 bytes.
#[derive(Default)]
pub struct PlainHasher {
	prefix: u64,
}

impl Hasher for PlainHasher {
	#[inline]
	fn finish(&self) -> u64 {
		self.prefix
	}

	#[inline]
	#[allow(unused_assignments)]
	fn write(&mut self, bytes: &[u8]) {
		debug_assert!(bytes.len() == 32);
		let mut bytes_ptr = bytes.as_ptr();
		let mut prefix_bytes = self.prefix.to_le_bytes();

		unroll! {
			for _i in 0..8 {
				unsafe {
					prefix_bytes[_i] ^= (*bytes_ptr ^ *bytes_ptr.offset(8)) ^ (*bytes_ptr.offset(16) ^ *bytes_ptr.offset(24));
					bytes_ptr = bytes_ptr.offset(1);
				}
			}
		}

		self.prefix = u64::from_le_bytes(prefix_bytes);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		let mut bytes = [32u8; 32];
		bytes[0] = 15;
		let mut hasher = PlainHasher::default();
		hasher.write(&bytes);
		assert_eq!(hasher.prefix, 47);
	}
}
