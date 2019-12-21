
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
