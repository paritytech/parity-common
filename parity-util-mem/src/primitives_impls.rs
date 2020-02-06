// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

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
