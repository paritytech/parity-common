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
use hex_prefix_encoding::hex_prefix_encode_substrate;
use hashdb::Hasher;
use super::TrieStream;
use parity_codec::Encode;

/// Codec-flavoured TrieStream
pub struct CodecTrieStream {
	buffer: Vec<u8>
}

const BRANCH_NODE:u8 = 0b01_00_0000;
const EMPTY_NODE:u8 = 0;
impl CodecTrieStream { }
impl TrieStream for CodecTrieStream {
	fn new() -> Self { Self {buffer: Vec::new() } }
	fn append_empty_data(&mut self) {
		self.buffer.push(EMPTY_NODE);
	}

	// TODO: why `Hasher` here; it was needed for rlp_triestream but why?
	fn append_leaf<H: Hasher>(&mut self, key: &[u8], value: &[u8]) {
		let mut hpe = hex_prefix_encode_substrate(key, true);
		self.buffer.push(hpe.next().expect("key is not empty; qed"));
		// TODO: I'd like to do `hpe.encode_to(&mut self.buffer);` here; need an `impl<'a> Encode for impl Iterator<Item = u8> + 'a`?
		hpe.collect::<Vec<u8>>().encode_to(&mut self.buffer);
		value.encode_to(&mut self.buffer);
	}
	fn begin_branch(&mut self) {
		println!("[begin_branch] pushing BRANCH_NODE: {}, {:#x?}, {:#010b}", BRANCH_NODE, BRANCH_NODE, BRANCH_NODE);
		self.buffer.push(BRANCH_NODE);
		println!("[begin_branch] buffer so far: {:#x?}", self.buffer);
		// TODO: I think this is wrong. I need to know how long the full branch node is I think and
		// this does not keep track of that.
	}
	fn append_value(&mut self, value: &[u8]) {
		value.encode_to(&mut self.buffer);
	}
	fn append_extension(&mut self, key: &[u8]) {
		let mut hpe = hex_prefix_encode_substrate(key, false);
		self.buffer.push(hpe.next().expect("key is not empty; qed"));
		hpe.collect::<Vec<u8>>().encode_to(&mut self.buffer);
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
	fn as_raw(&self) -> &[u8] { &self.buffer }
}
