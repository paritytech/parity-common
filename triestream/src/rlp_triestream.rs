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
use rlp::RlpStream;
use hashdb::Hasher;
use super::TrieStream;

/// RLP-flavoured TrieStream
pub struct RlpTrieStream {
	stream: RlpStream
}

impl RlpTrieStream {
	fn append_hashed<H: Hasher>(&mut self, data: &[u8]) -> &mut Self {
		// This is a hack to work around `append()` requiring `Encodable` – what is a better way?
		let mut s = RlpStream::new();
		s.encoder().encode_value(&H::hash(&data).as_ref());
		let rlp_val = s.out();
		self.stream.append_raw(&rlp_val, 1);
		self
	}
	// useful for debugging but not used otherwise
	pub fn as_raw(&self) -> &[u8] { &self.stream.as_raw() }
}

impl TrieStream for RlpTrieStream {
	fn new() -> Self { Self { stream: RlpStream::new() } }
	fn append_empty_data(&mut self) { self.stream.append_empty_data(); }
	fn begin_branch(&mut self) { self.stream.begin_list(17); }
	fn append_value(&mut self, value: &[u8]) {
		self.stream.append(&value);
	}
	fn append_extension(&mut self, key: &[u8]) {
		self.stream.begin_list(2);
		self.stream.append_iter(hex_prefix_encode(key, false));
	}
	fn append_substream<H: Hasher>(&mut self, other: Self) {
		let data = other.out();
		match data.len() {
			0...31 => {self.stream.append_raw(&data, 1);},
			_ => {self.append_hashed::<H>(&data);}
		};
	}
	// TODO: why is Hasher needed here?
	fn append_leaf(&mut self, key: &[u8], value: &[u8]) {
		self.stream.begin_list(2);
		// println!("[rlp_triestream, append_leaf] hpe'd key: {:#x?}", hex_prefix_encode(key, true).collect::<Vec<u8>>());
		self.stream.append_iter(hex_prefix_encode(key, true));
		// println!("[rlp_triestream, append_leaf] stream after appending key: {:#x?}", self.stream.as_raw());
		self.stream.append(&value);
	}

	fn out(self) -> Vec<u8> { self.stream.out() }
}
