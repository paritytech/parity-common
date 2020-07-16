// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Simple example, illustating usage of runtime wrapper.

use parity_runtime::Runtime;
use std::{thread::park_timeout, time::Duration};
use tokio::{fs::read_dir, stream::*};

/// Read current directory in a future, which is executed in the created runtime
fn main() {
	let runtime = Runtime::with_default_thread_count();
	runtime.executor().spawn_std(async move {
		let mut dirs = read_dir(".").await.unwrap();
		while let Some(dir) = dirs.try_next().await.expect("Error") {
			println!("{:?}", dir.path());
		}
	});
	let timeout = Duration::from_secs(3);
	park_timeout(timeout);
}
