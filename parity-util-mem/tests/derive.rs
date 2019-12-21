
#[test]
fn derive_smoky() {

	use parity_util_mem::{MallocSizeOf, MallocSizeOfExt};

	#[derive(MallocSizeOf)]
	struct Trivia {
		v: Vec<u8>,
	}

	let t = Trivia { v: vec![0u8; 1024] };

	assert_eq!(t.malloc_size_of(), 1024);
}

#[test]
fn derive_hashmap() {

	use parity_util_mem::{MallocSizeOf, MallocSizeOfExt};

	#[derive(MallocSizeOf, Default)]
	struct Trivia {
		hm: std::collections::HashMap<u64, Vec<u8>>,
	}

	let mut t = Trivia::default();

	t.hm.insert(1, vec![0u8; 2048]);

	assert_eq!(t.malloc_size_of(), 2088);
}
