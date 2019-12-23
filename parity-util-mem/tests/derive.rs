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

use parity_util_mem::{MallocSizeOf, MallocSizeOfExt};

#[test]
#[cfg(feature = "std")]
fn derive_vec() {
	#[derive(MallocSizeOf)]
	struct Trivia {
		v: Vec<u8>,
	}

	let t = Trivia { v: vec![0u8; 1024] };

	assert!(t.malloc_size_of() > 1000);
}

#[cfg(feature="std")]
#[test]
fn derive_hashmap() {
	#[derive(MallocSizeOf, Default)]
	struct Trivia {
		hm: std::collections::HashMap<u64, Vec<u8>>,
	}

	let mut t = Trivia::default();

	t.hm.insert(1, vec![0u8; 2048]);

	assert!(t.malloc_size_of() > 2000);
}

#[test]
#[cfg(feature = "std")]
fn derive_ignore() {
	#[derive(MallocSizeOf, Default)]
	struct Trivia {
		hm: std::collections::HashMap<u64, Vec<u8>>,
		#[ignore_malloc_size_of = "I don't like vectors"]
		v: Vec<u8>,
	}

	let mut t = Trivia::default();

	t.hm.insert(1, vec![0u8; 2048]);
	t.v = vec![0u8; 1024];
	assert!(t.malloc_size_of() < 3000);
}
