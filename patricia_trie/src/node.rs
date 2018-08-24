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

use elastic_array::ElasticArray36;
use nibbleslice::NibbleSlice;
use nibblevec::NibbleVec;
use hashdb::DBValue;

/// Partial node key type.
pub type NodeKey = ElasticArray36<u8>;

/// Type of node in the trie and essential information thereof.
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Node<'a> {
	/// Null trie node; could be an empty root or an empty branch entry.
	Empty,
	/// Leaf node; has key slice and value. Value may not be empty.
	Leaf(NibbleSlice<'a>, &'a [u8]),
	/// Extension node; has key slice and node data. Data may not be null.
	Extension(NibbleSlice<'a>, &'a [u8]),
	/// Branch node; has array of 16 child nodes (each possibly null) and an optional immediate node data.
	Branch([&'a [u8]; 16], Option<&'a [u8]>),
}

/// A Sparse (non mutable) owned vector struct to hold branch keys
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct BranchKeys {
	data: Vec<u8>,
	ubounds: [usize; 17],
}

impl<'a> From<[&'a [u8]; 16]> for BranchKeys {
	fn from(a: [&'a [u8]; 16]) -> Self {
		let mut data = Vec::with_capacity(a.iter().map(|inner| inner.len()).sum());
		let mut ubounds = [0; 17];
		for (inner, ub) in a.iter().zip(ubounds.iter_mut().skip(1)) {
			data.extend_from_slice(inner);
			*ub = data.len();
		}
		BranchKeys { data, ubounds }
	}
}

impl ::std::ops::Index<usize> for BranchKeys {
	type Output = [u8];
	fn index(&self, index: usize) -> &[u8] {
		assert!(index < 16);
		&self.data[self.ubounds[index]..self.ubounds[index + 1]]
	}
}

/// An owning node type. Useful for trie iterators.
#[derive(Debug, PartialEq, Eq)]
pub enum OwnedNode {
	/// Empty trie node.
	Empty,
	/// Leaf node: partial key and value.
	Leaf(NibbleVec, DBValue),
	/// Extension node: partial key and child node.
	Extension(NibbleVec, DBValue),
	/// Branch node: 16 children and an optional value.
	Branch(BranchKeys, Option<DBValue>),
}

impl<'a> From<Node<'a>> for OwnedNode {
	fn from(node: Node<'a>) -> Self {
		match node {
			Node::Empty => OwnedNode::Empty,
			Node::Leaf(k, v) => OwnedNode::Leaf(k.into(), DBValue::from_slice(v)),
			Node::Extension(k, child) => OwnedNode::Extension(k.into(), DBValue::from_slice(child)),
			Node::Branch(c, val) => OwnedNode::Branch(c.into(), val.map(DBValue::from_slice)),
		}
	}
}
