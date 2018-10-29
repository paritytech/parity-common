// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Threadpool forcing single thread synchronous usage.
//! Very likely to break existing code (implicit switch between asynchronous and synchronous
//! paradigm), because we do not ensure it runs only for unique thread configuration.
//!
//! A variant of this execution should allow running suspendable execution to run more use case.
//!

#[derive(Clone, Default)]
pub struct Builder(Option<String>);

pub struct ThreadPool(Option<String>);


impl Builder {
	pub fn new() -> Builder {
		Builder(None)
	}

	pub fn num_threads(self, _num_threads: usize) -> Builder {
		// do nothing (no error either (flexibility for usage in existing code)).
		self
	}


	pub fn thread_name(mut self, name: String) -> Builder {
		self.0 = Some(name);
		self
	}

	pub fn thread_stack_size(self, _size: usize) -> Builder {
		// unmanaged
		self
	}

	pub fn build(self) -> ThreadPool {
		ThreadPool(self.0)
	}

}

impl ThreadPool {

	pub fn new(num_threads: usize) -> ThreadPool {
		Builder::new().num_threads(num_threads).build()
	}

	pub fn with_name(name: String, num_threads: usize) -> ThreadPool {
		Builder::new().thread_name(name).num_threads(num_threads).build()
	}

	pub fn queued_count(&self) -> usize {
		// cannot queue with synch ex
		0
	}

	pub fn active_count(&self) -> usize {
		// cannot query running with non suspendable synch ex
		0
	}

	pub fn max_count(&self) -> usize {
		// single thread
		1
	}

	pub fn panic_count(&self) -> usize { 0 }

	pub fn set_num_threads(&mut self, _num_threads: usize) {	}

	pub fn join(&self) { }

	pub fn execute<F>(&self, job: F)
	where
		F: FnOnce() + Send + 'static,
	{
		job.call_once(())
	}

}
