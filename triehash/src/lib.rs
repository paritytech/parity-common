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
#[cfg(test)]
extern crate keccak_hasher;
#[cfg(test)]
extern crate parity_codec;

use std::collections::BTreeMap;
use std::cmp;
use std::fmt::Debug; // TODO: remove when done here along with all the `Debug` bounds

pub use hashdb::Hasher;

/// TODO: DOCUMENT!!!!
pub trait TrieStream {
	fn new() -> Self;
	fn append_empty_data(&mut self);
	fn begin_branch(&mut self, maybe_value: Option<&[u8]>, has_children: impl Iterator<Item = bool>);
	fn append_empty_child(&mut self) {}
	fn end_branch(&mut self, _value: Option<&[u8]>) {}
	fn append_leaf(&mut self, key: &[u8], value: &[u8]);
	fn append_extension(&mut self, key: &[u8]);
	fn append_substream<H: Hasher>(&mut self, other: Self);
	fn out(self) -> Vec<u8>;
}

fn shared_prefix_len<T: Eq>(first: &[T], second: &[T]) -> usize {
	first.iter()
		.zip(second.iter())
		.position(|(f, s)| f != s)
		.unwrap_or_else(|| cmp::min(first.len(), second.len()))
}

/// Generates a trie root hash for a vector of key-value tuples
///
/// ```rust
/// extern crate triehash;
/// extern crate keccak_hasher;
/// extern crate triestream;
/// use triehash::trie_root;
/// use keccak_hasher::KeccakHasher;
/// use triestream::RlpTrieStream;
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
pub fn trie_root<H, S, I, A, B>(input: I) -> H::Out where
	I: IntoIterator<Item = (A, B)>,
	A: AsRef<[u8]> + Ord + Debug,
	B: AsRef<[u8]> + Debug,
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
	H::hash(&stream.out())
}

//#[cfg(test)]	// consider feature="std"
pub fn unhashed_trie<H, S, I, A, B>(input: I) -> Vec<u8> where
	I: IntoIterator<Item = (A, B)> + Debug,
	A: AsRef<[u8]> + Ord + Debug,
	B: AsRef<[u8]> + Debug,
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

	// println!("as nibbles: {:#x?}", input);
	let mut stream = S::new();
	build_trie::<H, S, _, _>(&input, 0, &mut stream);
	stream.out()
}

/// Generates a key-hashed (secure) trie root hash for a vector of key-value tuples.
///
/// ```rust
/// extern crate triehash;
/// extern crate keccak_hasher;
/// extern crate triestream;
/// use triehash::sec_trie_root;
/// use keccak_hasher::KeccakHasher;
/// use triestream::RlpTrieStream;
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
pub fn sec_trie_root<H, S, I, A, B>(input: I) -> H::Out where
	I: IntoIterator<Item = (A, B)>,
	A: AsRef<[u8]> + Debug,
	B: AsRef<[u8]> + Debug,
	H: Hasher,
	H::Out: Ord,
	S: TrieStream,
{
	trie_root::<H, S, _, _, _>(input.into_iter().map(|(k, v)| (H::hash(k.as_ref()), v)))
}

/// Takes a slice of key/value tuples where the key is a slice of nibbles
/// and encodes it into the provided `Stream`.
// pub fn build_trie<H, S, A, B>(input: &[(A, B)], cursor: usize, stream: &mut S)
fn build_trie<H, S, A, B>(input: &[(A, B)], cursor: usize, stream: &mut S) where
	A: AsRef<[u8]> + Debug,
	B: AsRef<[u8]> + Debug,
	H: Hasher,
	S: TrieStream,
{
	match input.len() {
		// No input, just append empty data.
		0 => {
			// println!("[build_trie] no input; appending empty, cursor={}, stream={:?}", cursor, stream.as_raw());
			stream.append_empty_data()
		},
		// Leaf node; append the remainder of the key and the value. Done.
		1 => {
			// println!("[build_trie] appending leaf, cursor={}, stream={:?}, partial key={:?}", cursor, stream.as_raw(), &input[0].0.as_ref()[cursor..]);
			// stream.append_leaf::<H>(&input[0].0.as_ref()[cursor..], &input[0].1.as_ref() )
			stream.append_leaf(&input[0].0.as_ref()[cursor..], &input[0].1.as_ref() )
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
			// Add an extension node if the number of shared nibbles is greater
			// than what we saw on the last call (`cursor`): append the new part
			// of the path then recursively append the remainder of all items
			// who had this partial key.
			if shared_nibble_count > cursor {
				// println!("[build_trie] appending ext and recursing, cursor={}, stream={:?}, partial key={:?}", cursor, stream.as_raw(), &key[cursor..shared_nibble_count]);
				stream.append_extension(&key[cursor..shared_nibble_count]);
				build_trie_trampoline::<H, _, _, _>(input, shared_nibble_count, stream);
				// println!("[build_trie] returning after recursing, cursor={}, stream={:?}, partial key={:?}", cursor, stream.as_raw(), &key[cursor..shared_nibble_count]);
				return;
			}

			// We'll be adding a branch node because the path is as long as it gets.
			// First we need to figure out what entries this branch node will have...

			// We have a a value for exactly this key. Branch node will have a value
			// attached to it.
			let value = if cursor == key.len() { Some(value) } else { None };

			// We need to know how many keys each of the children account for.
			let mut shared_nibble_counts = [0usize; 16];
			{
				// Children keys begin at either index 1 or 0, depending on whether we have a value.
				let mut begin = match value { None => 0, _ => 1 };
				for i in 0..16 {
					shared_nibble_counts[i] = input[begin..].iter()
						.take_while(|(k, _)| k.as_ref()[cursor] == i as u8)
						.count();
					begin += shared_nibble_counts[i];
				}
			}

			// Put out the node header:
			stream.begin_branch(value, shared_nibble_counts.iter().map(|&n| n > 0));

			// Fill in each slot in the branch node. We don't need to bother with empty slots since they
			// were registered in the header.
			let mut begin = match value { None => 0, _ => 1 };
			for &count in &shared_nibble_counts {
				if count > 0 {
					// println!("[build_trie] branch slot {}; recursing with cursor={}, begin={}, shared nibbles={}, input={:?}", i, cursor, begin, shared_nibble_count, &input[begin..(begin + shared_nibble_count)]);
					build_trie_trampoline::<H, S, _, _>(&input[begin..(begin + count)], cursor + 1, stream);
					begin += count;
				} else {
					stream.append_empty_child();
				}
			}

			// println!("[build_trie] branch slot 17; cursor={}, appending value {:?}", cursor, value);
			stream.end_branch(value);

			// println!("[build_trie] ending branch node, cursor={}, stream={:?}", cursor, stream.as_raw());
		}
	}
}

fn build_trie_trampoline<H, S, A, B>(input: &[(A, B)], cursor: usize, stream: &mut S) where
	A: AsRef<[u8]> + Debug,
	B: AsRef<[u8]> + Debug,
	H: Hasher,
	S: TrieStream,
{
	let mut substream = S::new();
	build_trie::<H, _, _, _>(input, cursor, &mut substream);
	stream.append_substream::<H>(substream);
}
