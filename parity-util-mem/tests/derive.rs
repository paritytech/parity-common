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

#[test]
#[cfg(feature = "std")]
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
