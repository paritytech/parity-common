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
use hashdb::Hasher;
use super::TrieStream;
use parity_codec::Encode;
use std::iter::once;

/// Codec-flavoured TrieStream
pub struct CodecTrieStream {
	buffer: Vec<u8>
}

const LEAF_NODE_OFFSET: u8 = 128;
const BRANCH_NODE: u8 = 128;
const EXTENSION_NODE_OFFSET: u8 = 0;
const EMPTY_NODE: u8 = 0;
impl CodecTrieStream {
	// useful for debugging but not used otherwise
	pub fn as_raw(&self) -> &[u8] { &self.buffer }
}

/// Create a leaf/extension node, encoding a number of nibbles. Note that this
/// cannot handle a number of nibbles that is zero or greater than 127 and if
/// you attempt to do so *IT WILL PANIC*.
pub fn fuse_nibbles_node<'a>(nibbles: &'a [u8], leaf: bool) -> impl Iterator<Item = u8> + 'a {
	// There's currently no
	assert!(nibbles.len() > 0, "Attempt to fuse zero nibbles into a node: this breaks an interface requirement.");
	assert!(nibbles.len() < 128, "Attempt to fuse more than 127 nibbles into a node: this breaks an interface requirement.");
	let first_byte = if leaf { LEAF_NODE_OFFSET } else { EXTENSION_NODE_OFFSET } + nibbles.len() as u8;
	once(first_byte).chain(nibbles.chunks(2).map(|ch| ch[0] << 4 | if ch.len() == 2 { ch[1] } else { 0 }))
}

impl TrieStream for CodecTrieStream {
	fn new() -> Self { Self {buffer: Vec::new() } }
	fn append_empty_data(&mut self) {
		self.buffer.push(EMPTY_NODE);
	}

	fn append_leaf(&mut self, key: &[u8], value: &[u8]) {
		assert!(key.len() > 0, "Empty key for a leaf or extension would result in a redundant node; Merkle tries don't have redundant nodes; qed");
		assert!(key.len() < 128, "Trie code allows keys to be added with maximum 63 bytes; max key nibbles must be 126; qed");
		self.buffer.extend(fuse_nibbles_node(key, true));
		// TODO: I'd like to do `hpe.encode_to(&mut self.buffer);` here; need an `impl<'a> Encode for impl Iterator<Item = u8> + 'a`?
		value.encode_to(&mut self.buffer);
	}
	fn begin_branch(&mut self) {
		println!("[begin_branch] pushing BRANCH_NODE: {}, {:#x?}, {:#010b}", BRANCH_NODE, BRANCH_NODE, BRANCH_NODE);
		self.buffer.push(BRANCH_NODE);
		println!("[begin_branch] buffer so far: {:#x?}", self.buffer);
	}
	fn append_value(&mut self, value: &[u8]) {
		value.encode_to(&mut self.buffer);
	}
	fn append_extension(&mut self, key: &[u8]) {
		self.buffer.extend(fuse_nibbles_node(key, false));
	}
	fn append_substream<H: Hasher>(&mut self, other: Self) {
		let data = other.out();
		println!("[append_substream] START own buffer: {:x?}", self.buffer);
		println!("[append_substream] START other buffer: {:x?}", data);
		match data.len() {
			0...31 => {
				println!("[append_substream] appending data, because data.len() = {}", data.len());
				data.encode_to(&mut self.buffer)
			},
			_ => {
				println!("[append_substream] would have hashed, because data.len() = {}", data.len());
				data.encode_to(&mut self.buffer)
				// TODO: re-enable hashing before merging
				// let hash = H::hash(&data);
				// hash.as_ref().encode_to(&mut self.buffer)
			}
		}
	}

	fn out(self) -> Vec<u8> { self.buffer }
}
