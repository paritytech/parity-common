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

//! Generates trie root.
//!
//! This module should be used to generate trie root hash.

extern crate hashdb;
extern crate rlp;
#[cfg(test)]
extern crate keccak_hasher;
#[macro_use]
extern crate log;
#[cfg(test)]
extern crate env_logger;

use std::collections::BTreeMap;
use std::cmp;
use std::iter::once;
use std::fmt;
use hashdb::Hasher;

mod stream;
pub use stream::TrieStream;
pub use stream::RlpTrieStream; // TODO: test-only, or move to façade crate

fn shared_prefix_len<T: Eq>(first: &[T], second: &[T]) -> usize {
	first.iter()
		.zip(second.iter())
		.position(|(f, s)| f != s)
		.unwrap_or_else(|| cmp::min(first.len(), second.len()))
}

/// Generates a trie root hash for a vector of values
///
/// ```rust
/// extern crate triehash;
/// extern crate keccak_hasher;
/// extern crate rlp;
/// use triehash::{ordered_trie_root, RlpTrieStream};
/// use keccak_hasher::KeccakHasher;
///
/// fn main() {
/// 	let v = &["doe", "reindeer"];
/// 	let root = "e766d5d51b89dc39d981b41bda63248d7abce4f0225eefd023792a540bcffee3";
/// 	assert_eq!(ordered_trie_root::<KeccakHasher, RlpTrieStream, _>(v), root.into());
/// }
/// ```
pub fn ordered_trie_root<H, S, I>(input: I) -> H::Out
where
	I: IntoIterator + fmt::Debug,
	I::Item: AsRef<[u8]> + fmt::Debug,
	H: Hasher,
	H::Out: cmp::Ord,
	S: TrieStream,
{
	// TODO: uses `rlp::encode`
	trie_root::<H, S, _, _, _>(input.into_iter().enumerate().map(|(i, v)| (rlp::encode(&i), v)))
}

/// Generates a trie root hash for a vector of key-value tuples
///
/// ```rust
/// extern crate triehash;
/// extern crate keccak_hasher;
/// use triehash::{trie_root, RlpTrieStream};
/// use keccak_hasher::KeccakHasher;
///
/// fn main() {
/// 	let v = vec![
/// 		("doe", "reindeer"),
/// 		("dog", "puppy"),
/// 		("dogglesworth", "cat"),
/// 	];
///
/// 	let root = "8aad789dff2f538bca5d8ea56e8abe10f4c7ba3a5dea95fea4cd6e7c3a1168d3";
/// 	assert_eq!(trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(v), root.into());
/// }
/// ```
pub fn trie_root<H, S, I, A, B>(input: I) -> H::Out
	where I: IntoIterator<Item = (A, B)>,
		  A: AsRef<[u8]> + Ord + std::fmt::Debug,
		  B: AsRef<[u8]> + std::fmt::Debug,
		  H: Hasher,
		  S: TrieStream,
{

	// first put elements into btree to sort them and to remove duplicates
	let input = input
		.into_iter()
		.collect::<BTreeMap<_, _>>();

	let mut nibbles = Vec::with_capacity(input.keys().map(|k| k.as_ref().len()).sum::<usize>() * 2);
	let mut lens = Vec::with_capacity(input.len() + 1);
	lens.push(0);
	for k in input.keys() {
		for &b in k.as_ref() {
			nibbles.push(b >> 4);
			nibbles.push(b & 0x0F);
		}
		lens.push(nibbles.len());
	}

	// then move them to a vector
	let input = input.into_iter().zip(lens.windows(2))
		.map(|((_, v), w)| (&nibbles[w[0]..w[1]], v))
		.collect::<Vec<_>>();

	let mut stream = S::new();
	build_trie::<H, S, _, _>(&input, 0, &mut stream);
	trace!(target: "triehash", "[new, trie_root] Done building trie. Ready to flush.");
	H::hash(&stream.out())
}

/// Generates a key-hashed (secure) trie root hash for a vector of key-value tuples.
///
/// ```rust
/// extern crate triehash;
/// extern crate keccak_hasher;
/// use triehash::{sec_trie_root, RlpTrieStream};
/// use keccak_hasher::KeccakHasher;
///
/// fn main() {
/// 	let v = vec![
/// 		("doe", "reindeer"),
/// 		("dog", "puppy"),
/// 		("dogglesworth", "cat"),
/// 	];
///
/// 	let root = "d4cd937e4a4368d7931a9cf51686b7e10abb3dce38a39000fd7902a092b64585";
/// 	assert_eq!(sec_trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(v), root.into());
/// }
/// ```
pub fn sec_trie_root<H, S, I, A, B>(input: I) -> H::Out
where
	I: IntoIterator<Item = (A, B)> + fmt::Debug,
	A: AsRef<[u8]> + fmt::Debug,
	B: AsRef<[u8]> + fmt::Debug,
	H: Hasher,
	H::Out: Ord,
	S: TrieStream,
{
	trie_root::<H, S, _, _, _>(input.into_iter().map(|(k, v)| (H::hash(k.as_ref()), v)))
}

/// Hex-prefix Encoding. Encodes a payload and a flag. The high nibble of the first
/// bytes contains the flag; the lowest bit of the flag encodes the oddness of the
/// length and the second-lowest bit encodes whether the node is a value node. The
/// low nibble of the first byte is zero in the case of an even number of nibbles
/// in the payload, otherwise it is set to the first nibble of the payload.
/// All remaining nibbles (now an even number) fit properly into the remaining bytes.
///
/// The "termination marker" and "leaf-node" specifier are equivalent.
///
/// Input nibbles are in range `[0, 0xf]`.
///
/// ```markdown
///  [0,0,1,2,3,4,5]   0x10_01_23_45	// length is odd (7) so the lowest bit of the high nibble of the first byte is `1`; it is not a leaf node, so the second-lowest bit of the high nibble is 0; given it's an odd length, the lower nibble of the first byte is set to the first nibble. All in all we get 0b0001_000 (oddness) + 0b0000_0000 (is leaf?) + 0b0000_0000 = 0b0001_0000 = 0x10 and then we append the other nibbles
///  [0,1,2,3,4,5]     0x00_01_23_45	// length is even (6) and this is not a leaf node so the high nibble of the first byte is 0; the low nibble of the first byte is unused (0)
///  [1,2,3,4,5]       0x11_23_45   	// odd length, not leaf => high nibble of 1st byte is 0b0001 and low nibble of 1st byte is set to the first payload nibble (1) so all in all: 0b00010001, 0x11
///  [0,0,1,2,3,4]     0x00_00_12_34	// even length, not leaf => high nibble is 0 and the low nibble is unused so we get 0x00 and then the payload: 0x00_00_12…
///  [0,1,2,3,4]       0x10_12_34		// odd length, not leaf => oddness flag + first nibble (0) => 0x10
///  [1,2,3,4]         0x00_12_34
///  [0,0,1,2,3,4,5,T] 0x30_01_23_45	// odd length (7), leaf => high nibble of 1st byte is 0b0011; low nibble is set to 1st payload nibble so the first encoded byte is 0b0011_0000, i.e. 0x30
///  [0,0,1,2,3,4,T]   0x20_00_12_34	// even length (6), lead => high nibble of 1st byte is 0b0010; low nibble unused
///  [0,1,2,3,4,5,T]   0x20_01_23_45
///  [1,2,3,4,5,T]     0x31_23_45		// odd length (5), leaf => high nibble of 1st byte is 0b0011; low nibble of 1st byte is set to first payload nibble (1) so the 1st byte becomes 0b0011_0001, i.e. 0x31
///  [1,2,3,4,T]       0x20_12_34
/// ```
fn hex_prefix_encode<'a>(nibbles: &'a [u8], leaf: bool) -> impl Iterator<Item = u8> + 'a {
	let inlen = nibbles.len();
	let oddness_factor = inlen % 2;

	let first_byte = {
		let mut bits = ((inlen as u8 & 1) + (2 * leaf as u8)) << 4;
		if oddness_factor == 1 {
			bits += nibbles[0];
		}
		bits
	};
	once(first_byte).chain(nibbles[oddness_factor..].chunks(2).map(|ch| ch[0] << 4 | ch[1]))
}

/// Takes a slice of key/value tuples where the key is a slice of nibbles
/// and encodes it into the provided `Stream`.
fn build_trie<H, S, A, B>(input: &[(A, B)], cursor: usize, stream: &mut S)
where
	A: AsRef<[u8]> + std::fmt::Debug,
	B: AsRef<[u8]> + std::fmt::Debug,
	H: Hasher,
	S: TrieStream,
{
	trace!(target: "triehash", "[new] START with input nibbles: {:?}, length: {:?}, shared prefix len: {:?}", input, input.len(), cursor);

	match input.len() {
		// No input, just append empty data.
		0 => {
			stream.append_empty_data();
			trace!(target: "triehash", "[new] no input. END. stream={:x?}", stream.as_raw());
		},
		// Leaf node; append the remainder of the key and the value. Done.
		1 => {
			stream.append_leaf::<H>(&input[0].0.as_ref()[cursor..], &input[0].1.as_ref() );
			trace!(target: "triehash", "[new] Single item (leaf). END. stream={:x?}", stream.as_raw());
		},
		// We have multiple items in the input. We need to figure out if we
		// should add an extension node or a branch node.
		_ => {
			let (key, value) = (&input[0].0.as_ref(), input[0].1.as_ref());
			// Count the number of nibbles in the other elements that are
			// shared with the first key.
			// e.g. input = [ [1'7'3'10'12'13], [1'7'3'], [1'7'7'8'9'] ] => [1'7'] is common => 2
			let shared_nibble_count = input.iter().skip(1).fold(key.len(), |acc, &(ref k, _)| {
				cmp::min( shared_prefix_len(key, k.as_ref()), acc )
			});
			trace!(target: "triehash", "[new] Multiple items: {}. Length of prefix shared by all key nibbles: {}", input.len(), shared_nibble_count);
			// Add an extension node if the number of shared nibbles is greater
			// than what we saw on the last call (`cursor`): append the new part
			// of the path then recursively append the remainder of all items
			// who had this partial key.
			if shared_nibble_count > cursor {
				trace!(target: "triehash", "[new] {} nibbles are shared. We need an extension node. Current cursor: {}", shared_nibble_count, cursor);
				stream.append_extension(&key[cursor..shared_nibble_count]);
				trace!(target: "triehash", "[new] shared_prefix ({:?}) is longer than prefix len ({:?}); appending path {:x?} to stream", shared_nibble_count, cursor, &key[cursor..shared_nibble_count]);
				build_trie_trampoline::<H, _, _, _>(input, shared_nibble_count, stream);
				trace!(target: "triehash", "[new] back after recursing. END. stream: {:x?}", stream.as_raw());
				return;
			}
			trace!(target: "triehash", "[new] Nothing is shared. We need a branch node");
			trace!(target: "triehash", "[new] shared prefix ({:?}) is >= previous shared prefix ({})", shared_nibble_count, cursor);
			// Add a branch node because the path is as long as it gets. The branch
			// node has 17 entries, one for each possible nibble + 1 for data.
			stream.begin_branch();
			// If the length of the first key is equal to the current cursor, move
			// to next element.
			let mut begin = { if cursor == key.len() {1} else {0} };
			// Fill in each slot in the branch node: an empty node if the slot
			// is unoccupied, otherwise recurse and add more nodes.
			for i in 0..16 {
				// If we've reached the end of our input, fast-forward to the
				// end filling in the slots with empty nodes. The input is sorted
				// so we know there are no more elements we need to ponder.
				if begin >= input.len() {
					for _ in i..16 {
						stream.append_empty_data();
					}
					break;
				}
				// Count how many successive elements have same next nibble.
				let shared_nibble_count = input[begin..].iter()
					.inspect(|(k, v)| {
						trace!(target: "triehash", "    slot {}, input item: ({:?}, {:?}), pre_len'th key nibble, k[{}]: {} (in this slot? {})", i, k, v, cursor, k.as_ref()[cursor], k.as_ref()[cursor] == i)
					})
					.take_while(|(k, _)| k.as_ref()[cursor] == i)
					.count();
				// trace!(target: "triehash", "[new] slot {}: {} nibbles should go in this slot.", i, len);
				match shared_nibble_count {
					// If nothing is shared we're at the end of the path. Append
					// an empty node (and we'll append the value in the 17th slot
					// at the end of the method call).
					0 => stream.append_empty_data(),
					// If at least one successive element has the same nibble,
					// recurse and add more nodes.
					_ => {
						trace!(target: "triehash", "    slot {} {} successive elements have the same nibble. Recursing with {:?} and cursor {}", i, shared_nibble_count, &input[begin..(begin + shared_nibble_count)], cursor + 1);
						build_trie_trampoline::<H, S, _, _>(&input[begin..(begin + shared_nibble_count)], cursor + 1, stream);
						trace!(target: "triehash", "    slot {} Done recursing with {:?} and pre_len {}; stream={:x?}", i, &input[begin..(begin + shared_nibble_count)], cursor + 1, stream.as_raw());
					}
				}
				begin += shared_nibble_count;
			}
			trace!(target: "triehash", "[new] Done looping for branch node. Stream so far: {:x?}", stream.as_raw());
			if cursor == key.len() {
				trace!(target: "triehash", "[new] cursor {} == key.len() {}, so appending value={:x?}", cursor, key.len(), value);
				stream.append_value(value);
			} else {
				stream.append_empty_data();
			}
		}
	}
	trace!(target: "triehash", "[new] Done. stream={:x?}", stream.as_raw());
}

fn build_trie_trampoline<H, S, A, B>(input: &[(A, B)], cursor: usize, stream: &mut S)
where
	A: AsRef<[u8]> + std::fmt::Debug,
	B: AsRef<[u8]> + std::fmt::Debug,
	H: Hasher,
	S: TrieStream,
{
	trace!(target: "triehash", "[tra] START with input nibbles: {:?}, prefix length: {}", input, cursor);
	let mut substream = S::new();
	build_trie::<H, _, _, _>(input, cursor, &mut substream);
	stream.append_substream::<H>(substream);
	trace!(target: "triehash", "[tra] END. stream={:x?}", stream.as_raw());
}

#[cfg(test)]
mod tests {
	use super::{trie_root, shared_prefix_len, hex_prefix_encode};
	use super::{sec_trie_root};
	use keccak_hasher::KeccakHasher;
	use super::RlpTrieStream;

	use std::sync::{Once, ONCE_INIT};
    static INIT: Once = ONCE_INIT;

	fn setup() {
		INIT.call_once(|| { ::env_logger::init(); });
	}

	#[test]
	fn sec_trie_root_works() {
		setup();
		let v = vec![
			("doe", "reindeer"),
			("dog", "puppy"),
			("dogglesworth", "cat"),
		];
		assert_eq!(
			sec_trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(v.clone()),
			"d4cd937e4a4368d7931a9cf51686b7e10abb3dce38a39000fd7902a092b64585".into(),
		);
	}

	#[test]
	fn trie_root_works() {
		setup();
		let v = vec![
			("doe", "reindeer"),
			("dog", "puppy"),
			("dogglesworth", "cat"),
		];
		assert_eq!(
			trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(v),
			"8aad789dff2f538bca5d8ea56e8abe10f4c7ba3a5dea95fea4cd6e7c3a1168d3".into()
		);
		assert_eq!(
			trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(vec![
				(b"A", b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" as &[u8])
			]),
			"d23786fb4a010da3ce639d66d5e904a11dbc02746d1ce25029e53290cabf28ab".into()
		);
	}

	// TODO: add a test for ordered_trie_root which is essentially the only thing `parity-ethereum` uses

	#[test]
	fn test_hex_prefix_encode() {
		let v = vec![0, 0, 1, 2, 3, 4, 5];
		let e = vec![0x10, 0x01, 0x23, 0x45];
		let h = hex_prefix_encode(&v, false).collect::<Vec<_>>();
		assert_eq!(h, e);

		let v = vec![0, 1, 2, 3, 4, 5];
		let e = vec![0x00, 0x01, 0x23, 0x45];
		let h = hex_prefix_encode(&v, false).collect::<Vec<_>>();
		assert_eq!(h, e);

		let v = vec![0, 1, 2, 3, 4, 5];
		let e = vec![0x20, 0x01, 0x23, 0x45];
		let h = hex_prefix_encode(&v, true).collect::<Vec<_>>();
		assert_eq!(h, e);

		let v = vec![1, 2, 3, 4, 5];
		let e = vec![0x31, 0x23, 0x45];
		let h = hex_prefix_encode(&v, true).collect::<Vec<_>>();
		assert_eq!(h, e);

		let v = vec![1, 2, 3, 4];
		let e = vec![0x00, 0x12, 0x34];
		let h = hex_prefix_encode(&v, false).collect::<Vec<_>>();
		assert_eq!(h, e);

		let v = vec![4, 1];
		let e = vec![0x20, 0x41];
		let h = hex_prefix_encode(&v, true).collect::<Vec<_>>();
		assert_eq!(h, e);
	}

	#[test]
	fn test_triehash_out_of_order() {
		assert!(trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(vec![
			(vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
			(vec![0x81u8, 0x23], vec![0x81u8, 0x23]),
			(vec![0xf1u8, 0x23], vec![0xf1u8, 0x23]),
		]) ==
		trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(vec![
			(vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
			(vec![0xf1u8, 0x23], vec![0xf1u8, 0x23]), // last two tuples are swapped
			(vec![0x81u8, 0x23], vec![0x81u8, 0x23]),
		]));
	}

	#[test]
	fn test_shared_prefix() {
		let a = vec![1,2,3,4,5,6];
		let b = vec![4,2,3,4,5,6];
		assert_eq!(shared_prefix_len(&a, &b), 0);
	}

	#[test]
	fn test_shared_prefix2() {
		let a = vec![1,2,3,3,5];
		let b = vec![1,2,3];
		assert_eq!(shared_prefix_len(&a, &b), 3);
	}

	#[test]
	fn test_shared_prefix3() {
		let a = vec![1,2,3,4,5,6];
		let b = vec![1,2,3,4,5,6];
		assert_eq!(shared_prefix_len(&a, &b), 6);
	}
}
