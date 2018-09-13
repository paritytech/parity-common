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
use hashdb::Hasher;

mod stream;
pub use stream::TrieStream;
pub use stream::RlpTrieStream; // TODO: test-only, or move to façade crate

fn shared_prefix_len<T: Eq>(first: &[T], second: &[T]) -> usize {
	let len = cmp::min(first.len(), second.len());
	(0..len).take_while(|&i| first[i] == second[i]).count()
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
/// 	assert_eq!(ordered_trie_root::<KeccakHasher, RlpTrieStream, _, _>(v), root.into());
/// }
/// ```
pub fn ordered_trie_root<H, S, I, A>(input: I) -> H::Out
	where I: IntoIterator<Item = A>,
		  A: AsRef<[u8]> + std::fmt::Debug,
		  H: Hasher,
		  <H as hashdb::Hasher>::Out: cmp::Ord + rlp::Encodable,
		  S: TrieStream,
{
	let gen_input: Vec<_> = input
		// first put elements into btree to sort them by nibbles (key'd by index)
		// optimize it later
		.into_iter()
		.enumerate()
		.map(|(i, slice)| (rlp::encode(&i), slice))
		.collect::<BTreeMap<_, _>>()
		// then convert the key to nibbles and  move them to a vector of (k, v) tuples
		.into_iter()
		.map(|(k, v)| (as_nibbles(&k), v) )
		.collect();

	gen_trie_root::<H, S, _, _>(&gen_input)
}

/// Generates a trie root hash for a vector of key-value tuples
///
/// ```rust
/// extern crate triehash;
/// extern crate keccak_hasher;
/// extern crate rlp;
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
	where I: IntoIterator<Item = (A, B)> + std::fmt::Debug,
		  A: AsRef<[u8]> + Ord + std::fmt::Debug,
		  B: AsRef<[u8]> + std::fmt::Debug,
		  H: Hasher,
		  <H as hashdb::Hasher>::Out: cmp::Ord + rlp::Encodable,
		  S: TrieStream,
{
	trace!(target: "triehash", "[trie_root] input: {:?}", input);
	let gen_input: Vec<_> = input
		// first put elements into btree to sort them and to remove duplicates
		.into_iter()
		.collect::<BTreeMap<_, _>>()
		// then convert the key to nibbles and  move them to a vector of (k, v) tuples
		.into_iter()
		// .inspect(|x| trace!(target: "triehash", "[trie_root] element: {:?}, as nibble: {:?}", x, as_nibbles(x.0.as_ref())))
		.map(|(k, v)| (as_nibbles(k.as_ref()), v) )
		.collect();

	trace!(target: "triehash", "[trie_root] normalized input: {:?}", gen_input);
	gen_trie_root::<H, S, _, _>(&gen_input)
}
pub fn trie_root2<H, S, I, A, B>(input: I) -> H::Out
	where I: IntoIterator<Item = (A, B)> + std::fmt::Debug,
		  A: AsRef<[u8]> + Ord + std::fmt::Debug,
		  B: AsRef<[u8]> + std::fmt::Debug,
		  H: Hasher,
		  <H as hashdb::Hasher>::Out: cmp::Ord + rlp::Encodable,
		  S: TrieStream,
{
	let gen_input: Vec<_> = input
		.into_iter()
		.collect::<BTreeMap<_, _>>()
		.into_iter()
		.map(|(k, v)| (as_nibbles(k.as_ref()), v) )
		.collect();

	trace!(target: "triehash", "[trie_root2] sorted, nibbleized input: {:?}", gen_input);
	let mut stream = S::new();
	build_trie::<H, S, _, _>(&gen_input, &mut stream);
	H::hash(&stream.out())
}

/// Generates a key-hashed (secure) trie root hash for a vector of key-value tuples.
///
/// ```rust
/// extern crate triehash;
/// extern crate keccak_hasher;
/// extern crate rlp;
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
	where I: IntoIterator<Item = (A, B)>,
		  A: AsRef<[u8]> + std::fmt::Debug,
		  B: AsRef<[u8]> + std::fmt::Debug,
		  H: Hasher,
		  <H as hashdb::Hasher>::Out: cmp::Ord + rlp::Encodable,
		  S: TrieStream,
{
	let gen_input: Vec<_> = input
		// first put elements into btree to sort them and to remove duplicates
		.into_iter()
		.map(|(k, v)| (H::hash(k.as_ref()), v))
		.collect::<BTreeMap<_, _>>()
		// then convert the key to nibbles and  move them to a vector of (k, v) tuples
		.into_iter()
		.map(|(k, v)| (as_nibbles(k.as_ref()), v) )
		.collect();

	gen_trie_root::<H, S, _, _>(&gen_input)
}

fn gen_trie_root<H, S, A, B>(input: &[(A, B)]) -> H::Out
	where
		A: AsRef<[u8]> + std::fmt::Debug,
		B: AsRef<[u8]> + std::fmt::Debug,
		H: Hasher,
	  	<H as hashdb::Hasher>::Out: cmp::Ord + rlp::Encodable,
		S: TrieStream,
{
	// let mut stream = RlpTrieStream::new();
	let mut stream = S::new();
	hash256rlp::<H, S, _, _>(input, 0, &mut stream);
	H::hash(&stream.out())
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
pub(crate) fn hex_prefix_encode(nibbles: &[u8], leaf: bool) -> Vec<u8> {
	let inlen = nibbles.len();
	let oddness_factor = inlen % 2;
	let mut res = Vec::with_capacity(inlen/2 + 1);

	let first_byte = {
		let mut bits = ((inlen as u8 & 1) + (2 * leaf as u8)) << 4;
		if oddness_factor == 1 {
			bits += nibbles[0];
		}
		bits
	};

	res.push(first_byte);

	let mut offset = oddness_factor;
	while offset < inlen {
		let byte = (nibbles[offset] << 4) + nibbles[offset + 1];
		res.push(byte);
		offset += 2;
	}

	res
}

/// Converts slice of bytes to a vec of nibbles (reppresented as bytes).
/// Each input byte is converted to two new bytes containing the upper/lower
/// half of the original.
fn as_nibbles(bytes: &[u8]) -> Vec<u8> {
	let mut res = Vec::with_capacity(bytes.len() * 2);
	for i in 0..bytes.len() {
		let byte = bytes[i];
		// trace!(target:"triehash", "original byte:         {:#010b}", byte);
		res.push(byte >> 4);
		// trace!(target:"triehash", "right shifted 4 steps: {:#010b}", byte >> 4);
		res.push(byte & 0b1111);
		// trace!(target:"triehash", "anded:                 {:#010b}", byte & 0b1111);
	}
	res
}

/// Takes a vector of key/value tuples where the key is a slice of nibbles
/// and encodes it into the provided `Stream`.
fn hash256rlp<H, S, A, B>(input: &[(A, B)], pre_len: usize, stream: &mut S)
	where
		A: AsRef<[u8]> + std::fmt::Debug,
		B: AsRef<[u8]> + std::fmt::Debug,
		H: Hasher,
		<H as hashdb::Hasher>::Out: rlp::Encodable,
		S: TrieStream,
{
	let inlen = input.len();
	trace!(target: "triehash", "[hash256rlp] START with input nibbles: {:?}, length: {:?}, shared prefix len: {:?}", input, inlen, pre_len);
	// in case of empty slice, just append empty data
	if inlen == 0 {
		stream.append_empty_data();
		trace!(target: "triehash", "[hash256rlp] no input. END.");
		return;
	}

	// take slices
	let key: &[u8] = &input[0].0.as_ref();
	let value: &[u8] = &input[0].1.as_ref();

	// if the slice contains just one item, append the suffix of the key
	// and then append value
	if inlen == 1 {
		stream.begin_list(2);
		stream.append(&&*hex_prefix_encode(&key[pre_len..], true));
		stream.append(&value);
		trace!(target: "triehash", "[hash256rlp] single item. END.");
		return;
	}

	trace!(target: "triehash", "[hash256rlp] multiple items ({:?})", inlen);
	// get length of the longest shared prefix in slice keys
	let shared_prefix = input.iter()
		// skip first tuple
		.skip(1)
		// get minimum number of shared nibbles between first and each successive
		.fold(key.len(), | acc, &(ref k, _) | {
			let o = cmp::min(shared_prefix_len(key, k.as_ref()), acc);
			trace!(target: "triehash", "[hash256rlp] first key length: {:?}, k: {:?}, current acc: {:?}; shared prefix len: {:?}", key.len(), k, acc, o);
			o
		});

	// if shared prefix is higher than current prefix append its
	// new part of the key to the stream
	// then recursively append suffixes of all items who had this key
	if shared_prefix > pre_len {
		stream.begin_list(2);
		stream.append(&&*hex_prefix_encode(&key[pre_len..shared_prefix], false));
		trace!(target: "triehash", "[hash256rlp] shared_prefix ({:?}) is longer than prefix len ({:?}); appending path {:?} to stream", shared_prefix, pre_len, &key[pre_len..shared_prefix]);
		hash256aux::<H, S, _, _>(input, shared_prefix, stream);
		// trace!(target: "triehash", "[hash256rlp] back after recursing. Stream: {:?}. END.", stream);
		trace!(target: "triehash", "[hash256rlp] back after recursing. END.");
		return;
	}
	trace!(target: "triehash", "[hash256rlp] shared prefix ({:?}) is >= previous shared prefix ({})", shared_prefix, pre_len);
	// One item for every possible nibble/suffix + 1 for data
	stream.begin_list(17);

	// if first key len is equal to prefix_len, move to next element
	let mut begin = match pre_len == key.len() {
		true => {
			trace!(target: "triehash", "  starting list from 1 because pre_len == key.len() => {} == {}", pre_len, key.len());
			1},
		false => {
			trace!(target: "triehash", "  starting list from 0 because pre_len != key.len() => {} != {}", pre_len, key.len());
			0}
	};

	// iterate over all possible nibbles (4 bits => values between 0..16)
	for i in 0..16 {
		// count how many successive elements have same next nibble
		let len = match begin < input.len() {
			true => input[begin..].iter()
				.inspect(|(k, v)| {
					trace!(target: "triehash", "    slot {}, input item: ({:?}, {:?}), pre_len'th key nibble, k[{}]: {} (in this slot? {})", i, k, v, pre_len, k.as_ref()[pre_len], k.as_ref()[pre_len] == i)
				})
				.take_while(| (k, _) | k.as_ref()[pre_len] == i)
				.count(),
			false => 0
		};

		// trace!(target: "triehash", "    slot {} {} successive elements have the same nibble. Begin: {}", i, len, begin);

		// if at least 1 successive element has the same nibble
		// append their suffixes
		match len {
			0 => {
				trace!(target: "triehash", "    slot {} No successive element has the same nibble. Appending empty data.", i);
				stream.append_empty_data(); },
			_ => {
				trace!(target: "triehash", "    slot {} {} successive elements have the same nibble. Recursing with {:?}", i, len, &input[begin..(begin + len)]);
				hash256aux::<H, S, _, _>(&input[begin..(begin + len)], pre_len + 1, stream)}
		}
		begin += len;
	}
	trace!(target: "triehash", "[hash256rlp] Done looping");
	// if first key len is equal prefix, append its value
	match pre_len == key.len() {
		true => { stream.append(&value); },
		false => { stream.append_empty_data(); }
	};
}

fn hash256aux<H, S, A, B>(input: &[(A, B)], pre_len: usize, stream: &mut S)
	where
		A: AsRef<[u8]> + std::fmt::Debug,
		B: AsRef<[u8]> + std::fmt::Debug,
		H: Hasher,
		<H as hashdb::Hasher>::Out: rlp::Encodable,
		S: TrieStream,
{
	let mut s = S::new();
	trace!(target: "triehash", "[aux] START with input nibbles: {:?}, prefix length: {}", input, pre_len);
	if input.len() == 1 {
		trace!(target: "triehash", "[aux] single item. Appending k/v.");
		s.begin_list(2);
		s.append(&&*hex_prefix_encode(&input[0].0.as_ref()[pre_len..], true));
		s.append(&input[0].1.as_ref());
	} else {
		trace!(target: "triehash", "[aux] multiple items. Recursing.");
		hash256rlp::<H, S, _, _>(input, pre_len, &mut s);
	}
	let out = s.out();
	match out.len() {
		0...31 => {
			trace!(target: "triehash", "[aux] short output: {}, appending raw: {:?}", out.len(), &out);
			stream.append_raw(&out, 1)
		},
		_ => {
			trace!(target: "triehash", "[aux] long output: {}, appending hash", out.len());
			stream.append(&H::hash(&out))}
	};
	trace!(target: "triehash", "[aux] END.");
}

// fn encode<S: TrieStream>(input: Vec<(&[u8],&[u8])>) -> S {
// 	let mut s = S::new();
// 	let key: &[u8] = &input[0].0.as_ref();
// 	let value: &[u8] = &input[0].1.as_ref();
// /*
// 1. iterate over list of k/v
// 1.1 last value? append rest of the key and append value. Return.
// 2. find the number of nibbles that the first key have in common with the remaining keys
// 2.1 Yes? Append common nibbles to stream
// 2.2 No? Add new list of 17 items
// 3.
// */
// 	let shared_prefix = input.iter()
// 		// skip first tuple
// 		.skip(1)
// 		// get the number of shared nibbles between first item and each successive
// 		.fold(key.len(), | acc, &(ref k, _) | {
// 			let o = cmp::min(shared_prefix_len(key, k.as_ref()), acc);
// 			trace!(target: "triehash", "[hash256rlp] first key length: {:?}, k: {:?}, current acc: {:?}; shared prefix len: {:?}", key.len(), k, acc, o);
// 			o
// 		});
// 	s.begin_list(2); s.append(&&*hex_prefix_encode(&key[..shared_prefix], false));
// 	s
// }


fn build_trie<H, S, A, B>(input: &[(A, B)], stream: &mut S)
where
	A: AsRef<[u8]> + std::fmt::Debug,
	B: AsRef<[u8]> + std::fmt::Debug,
	H: Hasher,
	<H as hashdb::Hasher>::Out: rlp::Encodable,
	S: TrieStream,
{
	let input_length = input.len();
	match input_length {
		0 => stream.append_empty_data(),
		1 => stream.append_leaf(&&*hex_prefix_encode(input[0].0.as_ref(), true), &input[0].1.as_ref()),
		_ => {
			unreachable!();
		}
	}
}

#[test]
fn test_nibbles() {
	let v = vec![0x31, 0x23, 0x45];
	let e = vec![3, 1, 2, 3, 4, 5];
	assert_eq!(as_nibbles(&v), e);

	// A => 65 => 0x41 => [4, 1]
	let v: Vec<u8> = From::from("A");
	let e = vec![4, 1];
	assert_eq!(as_nibbles(&v), e);
}

#[cfg(test)]
mod tests {
	use super::{trie_root, trie_root2, shared_prefix_len, hex_prefix_encode};
	use keccak_hasher::KeccakHasher;
	use super::RlpTrieStream;

	use std::sync::{Once, ONCE_INIT};
    static INIT: Once = ONCE_INIT;

	fn setup() {
		INIT.call_once(|| { ::env_logger::init(); });
	}

	#[test]
	fn xlearn() {
		setup();
		let empty_input : Vec<(&[u8], &[u8])>= vec![];
		assert_eq!(
			trie_root2::<KeccakHasher, RlpTrieStream, _, _, _>(empty_input.clone()),
			trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(empty_input.clone()),
		);
		let input = vec![
			(&[0x5, 0x1], b"a" as &[u8]),
			(&[0x5, 0x3], b"c" as &[u8]),
			(&[0x5, 0x40], b"d" as &[u8]),
			(&[0x5, 0x2], b"b" as &[u8]), // out-of-order
			(&[0x5, 0x41], b"b" as &[u8]),
		];
		// let input = vec![(&[0xffu8, 0x02], b"a" as &[u8])];
		let _r = trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(input);
	}
	#[test]
	fn single_item_works() {
		setup();
		let input : Vec<(&[u8], &[u8])>= vec![(&[0x11], &[0x22])];
		assert_eq!(
			trie_root2::<KeccakHasher, RlpTrieStream, _, _, _>(input.clone()),
			trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(input.clone()),
		);
	}

	#[test]
	fn learn_nothing_shared() {
		setup();
		let input = vec![
			(&[0x16, 0x8], &[44]),
			(&[0x55, 0x7], &[13]),
		];
		let _r = trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(input);
	}

	#[test]
	fn test_hex_prefix_encode() {
		let v = vec![0, 0, 1, 2, 3, 4, 5];
		let e = vec![0x10, 0x01, 0x23, 0x45];
		let h = hex_prefix_encode(&v, false);
		assert_eq!(h, e);

		let v = vec![0, 1, 2, 3, 4, 5];
		let e = vec![0x00, 0x01, 0x23, 0x45];
		let h = hex_prefix_encode(&v, false);
		assert_eq!(h, e);

		let v = vec![0, 1, 2, 3, 4, 5];
		let e = vec![0x20, 0x01, 0x23, 0x45];
		let h = hex_prefix_encode(&v, true);
		assert_eq!(h, e);

		let v = vec![1, 2, 3, 4, 5];
		let e = vec![0x31, 0x23, 0x45];
		let h = hex_prefix_encode(&v, true);
		assert_eq!(h, e);

		let v = vec![1, 2, 3, 4];
		let e = vec![0x00, 0x12, 0x34];
		let h = hex_prefix_encode(&v, false);
		assert_eq!(h, e);

		let v = vec![4, 1];
		let e = vec![0x20, 0x41];
		let h = hex_prefix_encode(&v, true);
		assert_eq!(h, e);
	}

	#[test]
	fn simple_test() {
		assert_eq!(trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(vec![
			(b"A", b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" as &[u8])
		]), "d23786fb4a010da3ce639d66d5e904a11dbc02746d1ce25029e53290cabf28ab".into());
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
