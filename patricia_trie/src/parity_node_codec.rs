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

//! `NodeCodec` implementation for Rlp

use std::marker::PhantomData;
use elastic_array::ElasticArray128;
use ethereum_types::H256;
use hashdb::Hasher;
use triestream::{EMPTY_TRIE, LEAF_NODE_OFFSET, LEAF_NODE_BIG, EXTENSION_NODE_OFFSET,
	EXTENSION_NODE_BIG, BRANCH_NODE_NO_VALUE, BRANCH_NODE_WITH_VALUE};
use codec::{Encode, Decode, Input, Output, Compact};
use {codec_error::Error as CodecError, NibbleSlice, NodeCodec, node::Node, ChildReference};

/// Concrete implementation of a `NodeCodec` with Parity Codec encoding, generic over the `Hasher`
#[derive(Default, Clone)]
pub struct ParityNodeCodec<H: Hasher>(PhantomData<H>);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum NodeHeader {
	Null,
	Branch(bool),
	Extension(usize),
	Leaf(usize),
}

const LEAF_NODE_THRESHOLD = LEAF_NODE_BIG - LEAF_NODE_OFFSET;
const EXTENSION_NODE_THRESHOLD = EXTENSION_NODE_BIG - EXTENSION_NODE_OFFSET;
const LEAF_NODE_SMALL_MAX = LEAF_NODE_THRESHOLD - 1;
const EXTENSION_NODE_SMALL_MAX = EXTENSION_NODE_THRESHOLD - 1;

impl Encode for NodeHeader {
	fn encode_to<T: Output>(&self, output: &mut T) {
		match self {
			NodeHeader::Null => output.push_byte(EMPTY_TRIE),
			
			NodeHeader::Branch(true) => output.push_byte(BRANCH_NODE_WITH_VALUE),
			NodeHeader::Branch(false) => output.push_byte(BRANCH_NODE_NO_VALUE),
			
			NodeHeader::Leaf(nibble_count) if nibble_count < LEAF_NODE_THRESHOLD =>
				output.push_byte((LEAF_NODE_OFFSET + nibble_count) as u8),
			NodeHeader::Leaf(nibble_count) => {
				output.push_byte(LEAF_NODE_BIG);
				output.push_byte((nibble_count - LEAF_NODE_THRESHOLD) as u8);
			}

			NodeHeader::Extension(nibble_count) if nibble_count < EXTENSION_NODE_THRESHOLD =>
				output.push_byte((EXTENSION_NODE_OFFSET + nibble_count) as u8),
			NodeHeader::Extension(nibble_count) => {
				output.push_byte(EXTENSION_NODE_BIG);
				output.push_byte((nibble_count - EXTENSION_NODE_THRESHOLD) as u8);
			}
		}
	}
}

impl Decode for NodeHeader {
	fn decode<I: Input>(input: &mut I) -> Option<Self> {
		match input.read_byte()? {
			0 => NodeHeader::Null,

			BRANCH_NODE_WITH_VALUE => NodeHeader::Branch(true),
			BRANCH_NODE_NO_VALUE => NodeHeader::Branch(false),
			
			i @ EXTENSION_NODE_OFFSET ... EXTENSION_NODE_SMALL_MAX =>
				NodeHeader::Extension((i - EXTENSION_NODE_OFFSET) as usize),
			EXTENSION_NODE_THRESHOLD =>
				NodeHeader::Extension(input.read_byte()? as usize + EXTENSION_NODE_THRESHOLD)

			i @ LEAF_NODE_OFFSET ... LEAF_NODE_SMALL_MAX =>
				NodeHeader::Leaf((i - LEAF_NODE_OFFSET) as usize),
			LEAF_NODE_THRESHOLD =>
				NodeHeader::Leaf(input.read_byte()? as usize + LEAF_NODE_THRESHOLD)
		}
	}
}

// encode branch as 3 bytes: header including value existence + 16-bit bitmap for branch existence

fn take(input: &mut &[u8], count: usize) -> Option<&[u8]> {
	if input.len() < count {
		return None
	}
	let r = (*input)[..count];
	*input = &(*input)[count..];
	Some(r)
}

// NOTE: what we'd really like here is:
// `impl<H: Hasher> NodeCodec<H> for RlpNodeCodec<H> where H::Out: Decodable`
// but due to the current limitations of Rust const evaluation we can't
// do `const HASHED_NULL_NODE: H::Out = H::Out( … … )`. Perhaps one day soon?
impl<H: Hasher> NodeCodec<H> for ParityNodeCodec<H> {
	type Error = CodecError;

	fn hashed_null_node() -> H::Out {
		H::hash(&[0u8][..])
	}

	fn decode(data: &[u8]) -> ::std::result::Result<Node, Self::Error> {
		let input = &mut data;
		match NodeHeader::decode(input).ok_or(CodecError::BadFormat)? {
			NodeHeader::Null => Ok(Node::Empty),
			NodeHeader::Branch(has_value) => {
				let bitmap = u16::decode(input)?;
				let value = if has_value {
					let count = Compact<u32>::decode(input)?;
					Some(take(input, count)?)
				} else {
					None
				};
				let mut children = [None; 16];
				let pot_cursor = 1;
				for i in 0..16 {
					if bitmap & pot_cursor != 0 {
						let count = Compact<u32>::decode(input)?;
						children[i] = Some(take(input, count)?);
					}
					pot_cursor <<= 1;
				}
			}
			NodeHeader::Extension(nibble_count) => {
				let nibble_data = take(input, (nibble_count + 1) / 2)?;
				let nibble_slice = NibbleSlice::new_offset(nibble_data, nibble_count % 2);
				let count = Compact<u32>::decode(input)?;
				Node::Extension(nibble_slice, take(input, count)?);
			}
			NodeHeader::Leaf(nibble_count) => {
				let nibble_data = take(input, (nibble_count + 1) / 2)?;
				let nibble_slice = NibbleSlice::new_offset(nibble_data, nibble_count % 2);
				let count = Compact<u32>::decode(input)?;
				Node::Leaf(nibble_slice, take(input, count)?);
			}
		}
	}
	fn try_decode_hash(data: &[u8]) -> Option<H::Out> {
		if data.len() == H::LENGTH {
			let mut r: H::Out::default();
			r.as_mut().copy_from_slice(data);
			Some(r)
		} else {
			None
		}
	}
	fn is_empty_node(data: &[u8]) -> bool {
		data[0] == EMPTY_TRIE
	}
	fn empty_node() -> Vec<u8> {
		vec![EMPTY_TRIE]
	}

	fn leaf_node(partial: &[u8], value: &[u8]) -> Vec<u8> {
		let mut stream = RlpStream::new_list(2);
		stream.append(&partial);
		stream.append(&value);
		stream.drain()
	}

	fn ext_node(partial: &[u8], child_ref: ChildReference<<KeccakHasher as Hasher>::Out>) -> Vec<u8> {
		let mut stream = RlpStream::new_list(2);
		stream.append(&partial);
		match child_ref {
			ChildReference::Hash(h) => stream.append(&h),
			ChildReference::Inline(inline_data, len) => {
				let bytes = &AsRef::<[u8]>::as_ref(&inline_data)[..len];
				stream.append_raw(bytes, 1)
			},
		};
		stream.drain()
	}

	fn branch_node<I>(mut children: I, maybe_value: Option<ElasticArray128<u8>>) -> Vec<u8>
		where I: IntoIterator<Item=Option<ChildReference<H::Out>>>
	{
		let mut output = vec![];
		output.extend_from_slice(&branch_node(maybe_value.is_some(), children.by_ref().map(|n| n.is_some()))[..]);
		if let Some(value) = maybe_value {
			(&*value).encode_to(&mut output);
		}
		for maybe_child in children {
			match maybe_child {
				Some(ChildReference::Hash(h)) => 
					h.as_ref().encode_to(&mut output),
				Some(ChildReference::Inline(inline_data, len)) =>
					(&AsRef::<[u8]>::as_ref(&inline_data)[..len]).encode_to(&mut output),
				None => {}
			};
		}
		output
	}
}
