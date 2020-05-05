// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Performance timer with logging

use log::trace;
use std::time::Instant;

#[macro_export]
macro_rules! trace_time {
	($name: expr) => {
		let _timer = $crate::PerfTimer::new($name);
	};
}

/// Performance timer with logging. Starts measuring time in the constructor, prints
/// elapsed time in the destructor or when `stop` is called.
pub struct PerfTimer {
	name: &'static str,
	start: Instant,
}

impl PerfTimer {
	/// Create an instance with given name.
	pub fn new(name: &'static str) -> PerfTimer {
		PerfTimer { name, start: Instant::now() }
	}
}

impl Drop for PerfTimer {
	fn drop(&mut self) {
		let elapsed = self.start.elapsed();
		let ms = elapsed.as_millis();
		trace!(target: "perf", "{}: {:.2}ms", self.name, ms);
	}
}
