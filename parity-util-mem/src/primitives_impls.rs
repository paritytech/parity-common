// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation of `MallocSize` primitive types.

use primitive_types::{H160, H256, H512, U128, U256, U512};

malloc_size_of_is_0!(U128, U256, U512, H160, H256, H512);

#[cfg(test)]
mod tests {

	use primitive_types::H256;

	#[test]
	fn smoky() {
		let v = vec![H256::zero(), H256::zero()];

		assert!(crate::MallocSizeOfExt::malloc_size_of(&v) >= 64);
	}
}
