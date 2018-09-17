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

//! Defines the `TrieStream` trait used to build a byte-stream to calculate
//!  a trieroot. Comes in two flavours: rlp and substrate codec.

extern crate hashdb;
extern crate hex_prefix_encoding;
#[cfg(feature = "ethereum")]
extern crate rlp;

use hashdb::Hasher;

pub trait TrieStream {
	fn new() -> Self;
	fn append_empty_data(&mut self);
	fn begin_branch(&mut self);
	fn append_value(&mut self, value: &[u8]);
	fn append_leaf<H: Hasher>(&mut self, key: &[u8], value: &[u8]) where H: Hasher;
	fn append_extension(&mut self, key: &[u8]);
	fn append_substream<H: Hasher>(&mut self, other: Self);
	fn out(self) -> Vec<u8>;
	fn as_raw(&self) -> &[u8];

    fn encode(k: &usize) -> Vec<u8>; // Arrgh – `ordered_trie_root` enumerates and rlp-encodes items with `rlp::encode()` so need something similar here. :/
}

#[cfg(feature = "ethereum")]
mod rlp_triestream;
#[cfg(feature = "ethereum")]
pub use rlp_triestream::RlpTrieStream;
