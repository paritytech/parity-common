// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

//! Implementation of `MallocSize` for common types :
//! - etheureum types uint and fixed hash.
//! - elastic_array arrays
//! - parking_lot mutex structures

use super::{MallocSizeOf, MallocSizeOfOps};
use elastic_array::{
	ElasticArray1024, ElasticArray128, ElasticArray16, ElasticArray2, ElasticArray2048, ElasticArray256,
	ElasticArray32, ElasticArray36, ElasticArray4, ElasticArray512, ElasticArray64, ElasticArray8,
};
use ethereum_types::{Bloom, H128, H160, H256, H264, H32, H512, H520, H64, U128, U256, U512, U64};
use parking_lot::{Mutex, RwLock};

#[cfg(not(feature = "std"))]
use core as std;

#[cfg(feature = "std")]
malloc_size_of_is_0!(std::time::Instant);
malloc_size_of_is_0!(std::time::Duration);

malloc_size_of_is_0!(U64, U128, U256, U512, H32, H64, H128, H160, H256, H264, H512, H520, Bloom);

macro_rules! impl_elastic_array {
	($name: ident, $dummy: ident, $size: expr) => {
		impl<T> MallocSizeOf for $name<T>
		where
			T: MallocSizeOf,
		{
			fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
				self[..].size_of(ops)
			}
		}
	};
}

impl_elastic_array!(ElasticArray2, ElasticArray2Dummy, 2);
impl_elastic_array!(ElasticArray4, ElasticArray4Dummy, 4);
impl_elastic_array!(ElasticArray8, ElasticArray8Dummy, 8);
impl_elastic_array!(ElasticArray16, ElasticArray16Dummy, 16);
impl_elastic_array!(ElasticArray32, ElasticArray32Dummy, 32);
impl_elastic_array!(ElasticArray36, ElasticArray36Dummy, 36);
impl_elastic_array!(ElasticArray64, ElasticArray64Dummy, 64);
impl_elastic_array!(ElasticArray128, ElasticArray128Dummy, 128);
impl_elastic_array!(ElasticArray256, ElasticArray256Dummy, 256);
impl_elastic_array!(ElasticArray512, ElasticArray512Dummy, 512);
impl_elastic_array!(ElasticArray1024, ElasticArray1024Dummy, 1024);
impl_elastic_array!(ElasticArray2048, ElasticArray2048Dummy, 2048);

impl<T: MallocSizeOf> MallocSizeOf for Mutex<T> {
	fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
		(*self.lock()).size_of(ops)
	}
}

impl<T: MallocSizeOf> MallocSizeOf for RwLock<T> {
	fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
		self.read().size_of(ops)
	}
}
