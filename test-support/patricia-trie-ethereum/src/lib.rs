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

//! Façade crate for `patricia_trie` for Ethereum specific impls

pub extern crate patricia_trie as trie; // `pub` because we need to import this crate for the tests in `patricia_trie` and there were issues: https://gist.github.com/dvdplm/869251ee557a1b4bd53adc7c971979aa
extern crate elastic_array;
extern crate parity_bytes; // TODO: name changed; update upstream when `parity-common` is available
extern crate ethereum_types;
extern crate hashdb;
extern crate keccak_hasher;
extern crate rlp;
extern crate triehash;
extern crate hex_prefix_encoding;
#[cfg(test)]
extern crate memorydb;

mod rlp_node_codec;
mod rlp_triestream;

pub use rlp_node_codec::RlpNodeCodec;
pub use rlp_triestream::RlpTrieStream;

use ethereum_types::H256;
use keccak_hasher::KeccakHasher;
use rlp::DecoderError;

pub fn unhashed_trie(input: Vec<(&[u8], &[u8])>) -> Vec<u8> {
	triehash::unhashed_trie::<KeccakHasher, RlpTrieStream, _, _, _>(input)
}

pub fn trie_root(input: Vec<(&[u8], &[u8])>) -> H256 {
	triehash::trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(input)
}

pub fn sec_trie_root(input: Vec<(&[u8], &[u8])>) -> H256 {
	triehash::sec_trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(input)
}

/// Convenience type alias to instantiate a Keccak-flavoured `RlpNodeCodec`
pub type RlpCodec = RlpNodeCodec<KeccakHasher>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `TrieDB`
pub type TrieDB<'db> = trie::TrieDB<'db, KeccakHasher, RlpCodec>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `SecTrieDB`
pub type SecTrieDB<'db> = trie::SecTrieDB<'db, KeccakHasher, RlpCodec>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `FatDB`
pub type FatDB<'db> = trie::FatDB<'db, KeccakHasher, RlpCodec>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `TrieDBMut`
pub type TrieDBMut<'db> = trie::TrieDBMut<'db, KeccakHasher, RlpCodec>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `SecTrieDBMut`
pub type SecTrieDBMut<'db> = trie::SecTrieDBMut<'db, KeccakHasher, RlpCodec>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `FatDBMut`
pub type FatDBMut<'db> = trie::FatDBMut<'db, KeccakHasher, RlpCodec>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `TrieFactory`
pub type TrieFactory = trie::TrieFactory<KeccakHasher, RlpCodec>;

/// Convenience type alias for Keccak/Rlp flavoured trie errors
pub type TrieError = trie::TrieError<H256, DecoderError>;
/// Convenience type alias for Keccak/Rlp flavoured trie results
pub type Result<T> = trie::Result<T, H256, DecoderError>;

#[cfg(test)]
mod tests {
	use super::{RlpTrieStream, RlpNodeCodec};
	use triehash::{unhashed_trie, trie_root, sec_trie_root};
	use keccak_hasher::KeccakHasher;
	use memorydb::MemoryDB;
	use trie::{Hasher, DBValue, TrieMut, TrieDBMut};

	fn check_equivalent(input: Vec<(&[u8], &[u8])>) {
		let closed_form = trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(input.clone());
		let d = unhashed_trie::<KeccakHasher, RlpTrieStream, _, _, _>(input.clone());
		println!("Data: {:#x?}, {:#x?}, {:#x?}", d, KeccakHasher::hash(&d[..]), closed_form);
		let persistent = {
			let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
			let mut root = <KeccakHasher as Hasher>::Out::default();
			let mut t = TrieDBMut::<KeccakHasher, RlpNodeCodec<KeccakHasher>>::new(&mut memdb, &mut root);
			for (x, y) in input {
				t.insert(x, y).unwrap();
			}
			t.root().clone()
		};
		assert_eq!(closed_form, persistent);
	}

	#[test]
	fn empty_is_equivalent() {
		let input: Vec<(&[u8], &[u8])> = vec![];
		check_equivalent(input);
	}

	#[test]
	fn leaf_is_equivalent() {
		let input: Vec<(&[u8], &[u8])> = vec![(&[0xaa][..], &[0xbb][..])];
		check_equivalent(input);
	}

	#[test]
	fn branch_is_equivalent() {
		let input: Vec<(&[u8], &[u8])> = vec![(&[0xaa][..], &[0x10][..]), (&[0xba][..], &[0x11][..])];
		check_equivalent(input);
	}

	#[test]
	fn extension_and_branch_is_equivalent() {
		let input: Vec<(&[u8], &[u8])> = vec![(&[0xaa][..], &[0x10][..]), (&[0xab][..], &[0x11][..])];
		check_equivalent(input);
	}

	#[test]
	fn single_long_leaf_is_equivalent() {
		let input: Vec<(&[u8], &[u8])> = vec![(&[0xaa][..], &b"ABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABC"[..]), (&[0xba][..], &[0x11][..])];
		check_equivalent(input);
	}

	#[test]
	fn sec_trie_root_works() {
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
	fn learn_rlp_trie_empty() {
		let input: Vec<(&[u8], &[u8])> = vec![];
		let trie = unhashed_trie::<KeccakHasher, RlpTrieStream, _, _, _>(input);
		println!("[learn_rlp_trie_empty] 1st byte of trie:\n{:#010b}\n trie: {:#x?}", trie[0], trie );
		assert_eq!(trie, vec![0x80]);
	}

	#[test]
	fn learn_rlp_trie_single_item() {
		let input: Vec<(&[u8], &[u8])> = vec![(&[0x13], &[0x14])];
		let trie = unhashed_trie::<KeccakHasher, RlpTrieStream, _, _, _>(input);
		println!("[learn_rlp_trie_single_item] 1st byte of trie:\n{:#010b}\n trie: {:#x?}", trie[0], trie );
		assert_eq!(trie, vec![0xc4, 0x82, 0x20, 0x13, 0x14]);
		// The key, 0x13, as nibbles: [ 0x1, 0x3 ]
		// build_trie will call append_leaf with k/v: [ 0x1, 0x3 ], [0x14]
		// 	append_leaf will call rlp begin_list(2)
		// 		begin_list adds 0 to buffer - modified later when list is closed
		//	key is hpe'd: even length, leaf (terminated) => high nibble sets termination bit, low nibble is zero => 0b0010_0000 => 0x20 => 32
		// 	append_iter() is called with hpe byte + key byte => 0x20, 0x13; adds 0x80 + length of items (2) => 0x82
		//	buffer is now: 0, 0x82, 0x20, 0x13, 0x14
		//	append() adds the value bytes => 0x14 and closes the list: 0xc0 + length of payload => 0xc0 + 4
		// final buffer: 0xc4 0x82 0x20 0x13 0x14
	}

	#[test]
	fn learn_rlp_trie_single_item2() {
		let input = vec![(
			vec![0x12, 0x12, 0x12, 0x12, 0x13, 0x13], 	// key
			vec![0xff, 0xfe, 0xfd, 0xfc]				// val
		)];
		let trie = unhashed_trie::<KeccakHasher, RlpTrieStream, _, _, _>(input);
		// println!("[learn_rlp_trie_single_item] 1st byte of trie:\n{:#010b}\n trie: {:#x?}", trie[0], trie );
		assert_eq!(trie, vec![
			0xc0 + 13,	// list marker + 13 bytes long payload
			0x80 + 7,	// value marker + 7 bytes long payload
			0x20, 		// HPE byte
			0x12, 0x12, 0x12, 0x12, 0x13, 0x13,
			0x80 + 4, 	// value marker + 4 bytes long payload
			0xff, 0xfe, 0xfd, 0xfc
		]);
	}
}
