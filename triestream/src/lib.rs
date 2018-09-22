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
//! a trieroot. Comes in two flavours: rlp and substrate codec.

extern crate hashdb;
extern crate hex_prefix_encoding;
#[cfg(feature = "ethereum")]
extern crate rlp;

#[cfg(feature = "codec")]
extern crate parity_codec;

use hashdb::Hasher;

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

// The `RlpTrieStream` type could have gone into the `triehash-ethereum` crate
// over in `parity-common`. The reason for keeping it here under a feature flag
// is to make testing easier in `triehash`; with the type in `triehash-ethereum`
//  we'd end up with the same mess we have for the `patricia-trie` tests.
#[cfg(feature = "ethereum")]
mod rlp_triestream;
#[cfg(feature = "ethereum")]
pub use rlp_triestream::RlpTrieStream;

#[cfg(feature = "codec")]
pub mod codec_triestream;
#[cfg(feature = "codec")]
pub use codec_triestream::CodecTrieStream;
