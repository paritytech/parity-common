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
use hex_prefix_encoding::hex_prefix_encode;
use hashdb::Hasher;
use super::TrieStream;

/// Codec-flavoured TrieStream
pub struct CodecTrieStream {
	buffer: Vec<u8>
}

impl CodecTrieStream { }
impl TrieStream for CodecTrieStream {
	fn new() -> Self { Self {buffer: Vec::new() } }
	fn append_empty_data(&mut self) { }
	fn begin_branch(&mut self) { }
	fn append_value(&mut self, value: &[u8]) { }
	fn append_extension(&mut self, key: &[u8]) { }
	fn append_substream<H: Hasher>(&mut self, other: Self) {}
	fn append_leaf<H: Hasher>(&mut self, key: &[u8], value: &[u8]) {}
	fn out(self) -> Vec<u8> { self.buffer }
	fn as_raw(&self) -> &[u8] { &self.buffer }
}
