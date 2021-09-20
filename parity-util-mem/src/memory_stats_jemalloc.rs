// Copyright 2021 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub use tikv_jemalloc_ctl::Error;
use tikv_jemalloc_ctl::{epoch, stats};

#[derive(Clone)]
pub struct MemoryAllocationTracker {
	epoch: tikv_jemalloc_ctl::epoch_mib,
	allocated: stats::allocated_mib,
	resident: stats::resident_mib,
}

impl MemoryAllocationTracker {
	pub fn new() -> Result<Self, Error> {
		Ok(Self { epoch: epoch::mib()?, allocated: stats::allocated::mib()?, resident: stats::resident::mib()? })
	}

	pub fn snapshot(&self) -> Result<crate::MemoryAllocationSnapshot, Error> {
		// update stats by advancing the allocation epoch
		self.epoch.advance()?;

		let allocated: u64 = self.allocated.read()? as _;
		let resident: u64 = self.resident.read()? as _;
		Ok(crate::MemoryAllocationSnapshot { allocated, resident })
	}
}
