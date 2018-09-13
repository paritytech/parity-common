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

use std::borrow::Borrow;
use rlp::RlpStream;
use rlp::Encodable as RlpEncodable;
use hex_prefix_encode;

pub trait TrieStream {
	fn new() -> Self;
	fn append_empty_data(&mut self);
	// 17 item-long list
	// fn append_branch(&mut self, ...)
	// 2 item list with value
	fn append_leaf(&mut self, key: &[u8], value: &[u8]);
	// 2 item list with …?
	// fn append_extension(&mut self, ...)
	fn out(self) -> Vec<u8>;

	// legacy
	fn new_list(len: usize) -> Self;
	fn begin_list(&mut self, len: usize) -> &mut Self;
	fn append_raw<'a>(&'a mut self, bytes: &[u8], item_count: usize) -> &'a mut Self;
	fn append<'a, E>(&'a mut self, value: &E) -> &'a mut Self where E: RlpEncodable;
	fn append_list<'a, E, K>(&'a mut self, values: &[K]) -> &'a mut Self where E: RlpEncodable, K: Borrow<E>;
}

pub struct RlpTrieStream {
	stream: RlpStream
}

impl TrieStream for RlpTrieStream {
	fn new() -> Self { Self { stream: RlpStream::new() } }
	fn append_empty_data(&mut self) { self.stream.append_empty_data(); }
	fn append_leaf(&mut self, key: &[u8], value: &[u8]) {
		self.stream.begin_list(2);
		self.stream.append(&&*hex_prefix_encode(&key, true));
		self.stream.append(&value);
	}
	fn out(self) -> Vec<u8> {
		self.stream.out()
	}

	// legacy
	fn new_list(len: usize) -> Self { Self { stream: RlpStream::new_list(len) } }
	fn begin_list(&mut self, len: usize) -> &mut Self { self.stream.begin_list(len); self }
	fn append_raw<'a>(&'a mut self, bytes: &[u8], item_count: usize) -> &'a mut Self {
		self.stream.append_raw(bytes, item_count);
		self
	}
	fn append<'a, E>(&'a mut self, value: &E) -> &'a mut Self where E: RlpEncodable{
		self.stream.append(value);
		self
	}
	fn append_list<'a, E, K>(&'a mut self, values: &[K]) -> &'a mut Self where E: RlpEncodable, K: Borrow<E> {
		self.stream.append_list(values);
		self
	}
}
