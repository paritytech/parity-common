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

//! In-memory trie representation.

use super::{Result, TrieError, TrieMut};
use super::lookup::Lookup;
use super::node::Node as EncodedNode;
use node_codec::NodeCodec;
use super::{DBValue, node::NodeKey};

use bytes::ToPretty;
use hashdb::{HashDB, Hasher};
use nibbleslice::NibbleSlice;

use std::collections::{HashSet, VecDeque};
use std::marker::PhantomData;
use std::mem;
use std::ops::Index;
use std::{fmt::Debug, hash::Hash};

// For lookups into the Node storage buffer.
// This is deliberately non-copyable.
#[derive(Debug)]
struct StorageHandle(usize);

// Handles to nodes in the trie.
#[derive(Debug)]
enum NodeHandle<H> {
	/// Loaded into memory.
	InMemory(StorageHandle),
	/// Either a hash or an inline node
	Hash(H),
}

impl<H> From<StorageHandle> for NodeHandle<H> {
	fn from(handle: StorageHandle) -> Self {
		NodeHandle::InMemory(handle)
	}
}

fn empty_children<H>() -> Box<[Option<NodeHandle<H>>; 16]> {
	Box::new([
		None, None, None, None, None, None, None, None,
		None, None, None, None, None, None, None, None,
	])
}

/// Node types in the Trie.
#[derive(Debug)]
enum Node<H> {
	/// Empty node.
	Empty,
	/// A leaf node contains the end of a key and a value.
	/// This key is encoded from a `NibbleSlice`, meaning it contains
	/// a flag indicating it is a leaf.
	Leaf(NodeKey, DBValue),
	/// An extension contains a shared portion of a key and a child node.
	/// The shared portion is encoded from a `NibbleSlice` meaning it contains
	/// a flag indicating it is an extension.
	/// The child node is always a branch.
	Extension(NodeKey, NodeHandle<H>),
	/// A branch has up to 16 children and an optional value.
	Branch(Box<[Option<NodeHandle<H>>; 16]>, Option<DBValue>)
}

impl<O> Node<O>
where
	O: AsRef<[u8]> + AsMut<[u8]> + Default + Debug + PartialEq + Eq + Hash + Send + Sync + Clone + Copy
{
	// load an inline node into memory or get the hash to do the lookup later.
	fn inline_or_hash<C, H>(
		node: &[u8],
		db: &HashDB<H, DBValue>,
		storage: &mut NodeStorage<H::Out>
	) -> NodeHandle<H::Out>
	where
		C: NodeCodec<H>,
		H: Hasher<Out = O>,
	{
		C::try_decode_hash(&node)
			.map(NodeHandle::Hash)
			.unwrap_or_else(|| {
				let child = Node::from_encoded::<C, H>(node, db, storage);
				NodeHandle::InMemory(storage.alloc(Stored::New(child)))
			})
	}

	// decode a node from encoded bytes without getting its children.
	fn from_encoded<C, H>(data: &[u8], db: &HashDB<H, DBValue>, storage: &mut NodeStorage<H::Out>) -> Self
	where C: NodeCodec<H>,
		  H: Hasher<Out = O>,
	{
		match C::decode(data).expect("encoded bytes read from db; qed") {
			EncodedNode::Empty => Node::Empty,
			EncodedNode::Leaf(k, v) => Node::Leaf(k.encoded(true), DBValue::from_slice(&v)),
			EncodedNode::Extension(key, cb) => {
				Node::Extension(
					key.encoded(false),
					Self::inline_or_hash::<C, H>(cb, db, storage))
			}
			EncodedNode::Branch(ref encoded_children, val) => {
				let mut child = |i:usize| {
					encoded_children[i].map(|data|
						Self::inline_or_hash::<C, H>(data, db, storage)
					)
				};

				let children = Box::new([
					child(0), child(1), child(2), child(3),
					child(4), child(5), child(6), child(7),
					child(8), child(9), child(10), child(11),
					child(12), child(13), child(14), child(15),
				]);

				Node::Branch(children, val.map(DBValue::from_slice))
			}
		}
	}

	// TODO: parallelize
	fn into_encoded<F, C, H>(self, mut child_cb: F) -> Vec<u8>
	where
		C: NodeCodec<H>,
		F: FnMut(NodeHandle<H::Out>) -> ChildReference<H::Out>,
		H: Hasher<Out = O>,
	{
		match self {
			Node::Empty => C::empty_node(),
			Node::Leaf(partial, value) => C::leaf_node(&partial, &value),
			Node::Extension(partial, child) => C::ext_node(&partial, child_cb(child)),
			Node::Branch(mut children, value) => {
				C::branch_node(
					// map the `NodeHandle`s from the Branch to `ChildReferences`
					children.iter_mut()
						.map(Option::take)
						.map(|maybe_child|
							maybe_child.map(|child| child_cb(child))
						),
					value
				)
			}
		}
	}
}

// post-inspect action.
enum Action<H> {
	// Replace a node with a new one.
	Replace(Node<H>),
	// Restore the original node. This trusts that the node is actually the original.
	Restore(Node<H>),
	// if it is a new node, just clears the storage.
	Delete,
}

// post-insert action. Same as action without delete
enum InsertAction<H> {
	// Replace a node with a new one.
	Replace(Node<H>),
	// Restore the original node.
	Restore(Node<H>),
}

impl<H> InsertAction<H> {
	fn into_action(self) -> Action<H> {
		match self {
			InsertAction::Replace(n) => Action::Replace(n),
			InsertAction::Restore(n) => Action::Restore(n),
		}
	}

	// unwrap the node, disregarding replace or restore state.
	fn unwrap_node(self) -> Node<H> {
		match self {
			InsertAction::Replace(n) | InsertAction::Restore(n) => n,
		}
	}
}

// What kind of node is stored here.
enum Stored<H> {
	// A new node.
	New(Node<H>),
	// A cached node, loaded from the DB.
	Cached(Node<H>, H),
}

/// Used to build a collection of child nodes from a collection of `NodeHandle`s
pub enum ChildReference<HO> { // `HO` is e.g. `H256`, i.e. the output of a `Hasher`
	Hash(HO),
	Inline(HO, usize), // usize is the length of the node data we store in the `H::Out`
}

/// Compact and cache-friendly storage for Trie nodes.
struct NodeStorage<H> {
	nodes: Vec<Stored<H>>,
	free_indices: VecDeque<usize>,
}

impl<H> NodeStorage<H> {
	/// Create a new storage.
	fn empty() -> Self {
		NodeStorage {
			nodes: Vec::new(),
			free_indices: VecDeque::new(),
		}
	}

	/// Allocate a new node in the storage.
	fn alloc(&mut self, stored: Stored<H>) -> StorageHandle {
		if let Some(idx) = self.free_indices.pop_front() {
			self.nodes[idx] = stored;
			StorageHandle(idx)
		} else {
			self.nodes.push(stored);
			StorageHandle(self.nodes.len() - 1)
		}
	}

	/// Remove a node from the storage, consuming the handle and returning the node.
	fn destroy(&mut self, handle: StorageHandle) -> Stored<H> {
		let idx = handle.0;

		self.free_indices.push_back(idx);
		mem::replace(&mut self.nodes[idx], Stored::New(Node::Empty))
	}
}

impl<'a, H> Index<&'a StorageHandle> for NodeStorage<H> {
	type Output = Node<H>;

	fn index(&self, handle: &'a StorageHandle) -> &Node<H> {
		match self.nodes[handle.0] {
			Stored::New(ref node) => node,
			Stored::Cached(ref node, _) => node,
		}
	}
}

/// A `Trie` implementation using a generic `HashDB` backing database.
///
/// Use it as a `TrieMut` trait object. You can use `db()` to get the backing database object.
/// Note that changes are not committed to the database until `commit` is called.
/// Querying the root or dropping the trie will commit automatically.
///
/// # Example
/// ```
/// extern crate patricia_trie as trie;
/// extern crate patricia_trie_ethereum as ethtrie;
/// extern crate hashdb;
/// extern crate keccak_hash;
/// extern crate keccak_hasher;
/// extern crate memorydb;
/// extern crate ethereum_types;
///
/// use keccak_hash::KECCAK_NULL_RLP;
/// use ethtrie::{TrieDBMut, trie::TrieMut};
/// use trie::DBValue;
/// use keccak_hasher::KeccakHasher;
/// use memorydb::*;
/// use ethereum_types::H256;
///
/// fn main() {
///   let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
///   let mut root = H256::new();
///   let mut t = TrieDBMut::new(&mut memdb, &mut root);
///   assert!(t.is_empty());
///   assert_eq!(*t.root(), KECCAK_NULL_RLP);
///   t.insert(b"foo", b"bar").unwrap();
///   assert!(t.contains(b"foo").unwrap());
///   assert_eq!(t.get(b"foo").unwrap().unwrap(), DBValue::from_slice(b"bar"));
///   t.remove(b"foo").unwrap();
///   assert!(!t.contains(b"foo").unwrap());
/// }
/// ```
pub struct TrieDBMut<'a, H, C>
where
	H: Hasher + 'a,
	C: NodeCodec<H>
{
	storage: NodeStorage<H::Out>,
	db: &'a mut HashDB<H, DBValue>,
	root: &'a mut H::Out,
	root_handle: NodeHandle<H::Out>,
	death_row: HashSet<H::Out>,
	/// The number of hash operations this trie has performed.
	/// Note that none are performed until changes are committed.
	hash_count: usize,
	marker: PhantomData<C>, // TODO: rpheimer: "we could have the NodeCodec trait take &self to its methods and then we don't need PhantomData. we can just store an instance of C: NodeCodec in the trie struct. If it's a ZST it won't have any additional overhead anyway"
}

impl<'a, H, C> TrieDBMut<'a, H, C>
where
	H: Hasher,
	C: NodeCodec<H>
{
	/// Create a new trie with backing database `db` and empty `root`.
	pub fn new(db: &'a mut HashDB<H, DBValue>, root: &'a mut H::Out) -> Self {
		*root = C::hashed_null_node();
		let root_handle = NodeHandle::Hash(C::hashed_null_node());

		TrieDBMut {
			storage: NodeStorage::empty(),
			db: db,
			root: root,
			root_handle: root_handle,
			death_row: HashSet::new(),
			hash_count: 0,
			marker: PhantomData,
		}
	}

	/// Create a new trie with the backing database `db` and `root.
	/// Returns an error if `root` does not exist.
	pub fn from_existing(db: &'a mut HashDB<H, DBValue>, root: &'a mut H::Out) -> Result<Self, H::Out, C::Error> {
		if !db.contains(root) {
			return Err(Box::new(TrieError::InvalidStateRoot(*root)));
		}

		let root_handle = NodeHandle::Hash(*root);
		Ok(TrieDBMut {
			storage: NodeStorage::empty(),
			db: db,
			root: root,
			root_handle: root_handle,
			death_row: HashSet::new(),
			hash_count: 0,
			marker: PhantomData,
		})
	}
	/// Get the backing database.
	pub fn db(&self) -> &HashDB<H, DBValue> {
		self.db
	}

	/// Get the backing database mutably.
	pub fn db_mut(&mut self) -> &mut HashDB<H, DBValue> {
		self.db
	}

	// cache a node by hash
	fn cache(&mut self, hash: H::Out) -> Result<StorageHandle, H::Out, C::Error> {
		let node_encoded = self.db.get(&hash).ok_or_else(|| Box::new(TrieError::IncompleteDatabase(hash)))?;
		let node = Node::from_encoded::<C, H>(
			&node_encoded,
			&*self.db,
			&mut self.storage
		);
		Ok(self.storage.alloc(Stored::Cached(node, hash)))
	}

	// inspect a node, choosing either to replace, restore, or delete it.
	// if restored or replaced, returns the new node along with a flag of whether it was changed.
	fn inspect<F>(&mut self, stored: Stored<H::Out>, inspector: F) -> Result<Option<(Stored<H::Out>, bool)>, H::Out, C::Error>
	where F: FnOnce(&mut Self, Node<H::Out>) -> Result<Action<H::Out>, H::Out, C::Error> {
		Ok(match stored {
			Stored::New(node) => match inspector(self, node)? {
				Action::Restore(node) => Some((Stored::New(node), false)),
				Action::Replace(node) => Some((Stored::New(node), true)),
				Action::Delete => None,
			},
			Stored::Cached(node, hash) => match inspector(self, node)? {
				Action::Restore(node) => Some((Stored::Cached(node, hash), false)),
				Action::Replace(node) => {
					self.death_row.insert(hash);
					Some((Stored::New(node), true))
				}
				Action::Delete => {
					self.death_row.insert(hash);
					None
				}
			},
		})
	}

	// walk the trie, attempting to find the key's node.
	fn lookup<'x, 'key>(&'x self, mut partial: NibbleSlice<'key>, handle: &NodeHandle<H::Out>) -> Result<Option<DBValue>, H::Out, C::Error>
		where 'x: 'key
	{
		let mut handle = handle;
		loop {
			let (mid, child) = match *handle {
				NodeHandle::Hash(ref hash) => return Lookup{
					db: &*self.db,
					query: DBValue::from_slice,
					hash: hash.clone(),
					marker: PhantomData::<C>,
				}.look_up(partial),
				NodeHandle::InMemory(ref handle) => match self.storage[handle] {
					Node::Empty => return Ok(None),
					Node::Leaf(ref key, ref value) => {
						if NibbleSlice::from_encoded(key).0 == partial {
							return Ok(Some(DBValue::from_slice(value)));
						} else {
							return Ok(None);
						}
					}
					Node::Extension(ref slice, ref child) => {
						let slice = NibbleSlice::from_encoded(slice).0;
						if partial.starts_with(&slice) {
							(slice.len(), child)
						} else {
							return Ok(None);
						}
					}
					Node::Branch(ref children, ref value) => {
						if partial.is_empty() {
							return Ok(value.as_ref().map(|v| DBValue::from_slice(v)));
						} else {
							let idx = partial.at(0);
							match children[idx as usize].as_ref() {
								Some(child) => (1, child),
								None => return Ok(None),
							}
						}
					}
				}
			};

			partial = partial.mid(mid);
			handle = child;
		}
	}

	/// insert a key-value pair into the trie, creating new nodes if necessary.
	fn insert_at(&mut self, handle: NodeHandle<H::Out>, partial: NibbleSlice, value: DBValue, old_val: &mut Option<DBValue>) -> Result<(StorageHandle, bool), H::Out, C::Error> {
		let h = match handle {
			NodeHandle::InMemory(h) => h,
			NodeHandle::Hash(h) => self.cache(h)?,
		};
		let stored = self.storage.destroy(h);
		let (new_stored, changed) = self.inspect(stored, move |trie, stored| {
			trie.insert_inspector(stored, partial, value, old_val).map(|a| a.into_action())
		})?.expect("Insertion never deletes.");

		Ok((self.storage.alloc(new_stored), changed))
	}

	/// the insertion inspector.
	fn insert_inspector(&mut self, node: Node<H::Out>, partial: NibbleSlice, value: DBValue, old_val: &mut Option<DBValue>) -> Result<InsertAction<H::Out>, H::Out, C::Error> {
		trace!(target: "trie", "augmented (partial: {:?}, value: {:?})", partial, value.pretty());

		Ok(match node {
			Node::Empty => {
				trace!(target: "trie", "empty: COMPOSE");
				InsertAction::Replace(Node::Leaf(partial.encoded(true), value))
			}
			Node::Branch(mut children, stored_value) => {
				trace!(target: "trie", "branch: ROUTE,AUGMENT");

				if partial.is_empty() {
					let unchanged = stored_value.as_ref() == Some(&value);
					let branch = Node::Branch(children, Some(value));
					*old_val = stored_value;

					match unchanged {
						true => InsertAction::Restore(branch),
						false => InsertAction::Replace(branch),
					}
				} else {
					let idx = partial.at(0) as usize;
					let partial = partial.mid(1);
					if let Some(child) = children[idx].take() {
						// original had something there. recurse down into it.
						let (new_child, changed) = self.insert_at(child, partial, value, old_val)?;
						children[idx] = Some(new_child.into());
						if !changed {
							// the new node we composed didn't change. that means our branch is untouched too.
							return Ok(InsertAction::Restore(Node::Branch(children, stored_value)));
						}
					} else {
						// original had nothing there. compose a leaf.
						let leaf = self.storage.alloc(Stored::New(Node::Leaf(partial.encoded(true), value)));
						children[idx] = Some(leaf.into());
					}

					InsertAction::Replace(Node::Branch(children, stored_value))
				}
			}
			Node::Leaf(encoded, stored_value) => {
				let existing_key = NibbleSlice::from_encoded(&encoded).0;
				let cp = partial.common_prefix(&existing_key);
				if cp == existing_key.len() && cp == partial.len() {
					trace!(target: "trie", "equivalent-leaf: REPLACE");
					// equivalent leaf.
					let unchanged = stored_value == value;
					*old_val = Some(stored_value);

					match unchanged {
						// unchanged. restore
						true => InsertAction::Restore(Node::Leaf(encoded.clone(), value)),
						false => InsertAction::Replace(Node::Leaf(encoded.clone(), value)),
					}
				} else if cp == 0 {
					trace!(target: "trie", "no-common-prefix, not-both-empty (exist={:?}; new={:?}): TRANSMUTE,AUGMENT", existing_key.len(), partial.len());

					// one of us isn't empty: transmute to branch here
					let mut children = empty_children();
					let branch = if existing_key.is_empty() {
						// always replace since branch isn't leaf.
						Node::Branch(children, Some(stored_value))
					} else {
						let idx = existing_key.at(0) as usize;
						let new_leaf = Node::Leaf(existing_key.mid(1).encoded(true), stored_value);
						children[idx] = Some(self.storage.alloc(Stored::New(new_leaf)).into());

						Node::Branch(children, None)
					};

					// always replace because whatever we get out here is not the branch we started with.
					let branch_action = self.insert_inspector(branch, partial, value, old_val)?.unwrap_node();
					InsertAction::Replace(branch_action)
				} else if cp == existing_key.len() {
					trace!(target: "trie", "complete-prefix (cp={:?}): AUGMENT-AT-END", cp);

					// fully-shared prefix for an extension.
					// make a stub branch and an extension.
					let branch = Node::Branch(empty_children(), Some(stored_value));
					// augment the new branch.
					let branch = self.insert_inspector(branch, partial.mid(cp), value, old_val)?.unwrap_node();

					// always replace since we took a leaf and made an extension.
					let branch_handle = self.storage.alloc(Stored::New(branch)).into();
					InsertAction::Replace(Node::Extension(existing_key.encoded(false), branch_handle))
				} else {
					trace!(target: "trie", "partially-shared-prefix (exist={:?}; new={:?}; cp={:?}): AUGMENT-AT-END", existing_key.len(), partial.len(), cp);

					// partially-shared prefix for an extension.
					// start by making a leaf.
					let low = Node::Leaf(existing_key.mid(cp).encoded(true), stored_value);

					// augment it. this will result in the Leaf -> cp == 0 routine,
					// which creates a branch.
					let augmented_low = self.insert_inspector(low, partial.mid(cp), value, old_val)?.unwrap_node();

					// make an extension using it. this is a replacement.
					InsertAction::Replace(Node::Extension(
						existing_key.encoded_leftmost(cp, false),
						self.storage.alloc(Stored::New(augmented_low)).into()
					))
				}
			}
			Node::Extension(encoded, child_branch) => {
				let existing_key = NibbleSlice::from_encoded(&encoded).0;
				let cp = partial.common_prefix(&existing_key);
				if cp == 0 {
					trace!(target: "trie", "no-common-prefix, not-both-empty (exist={:?}; new={:?}): TRANSMUTE,AUGMENT", existing_key.len(), partial.len());

					// partial isn't empty: make a branch here
					// extensions may not have empty partial keys.
					assert!(!existing_key.is_empty());
					let idx = existing_key.at(0) as usize;

					let mut children = empty_children();
					children[idx] = if existing_key.len() == 1 {
						// direct extension, just replace.
						Some(child_branch)
					} else {
						// more work required after branching.
						let ext = Node::Extension(existing_key.mid(1).encoded(false), child_branch);
						Some(self.storage.alloc(Stored::New(ext)).into())
					};

					// continue inserting.
					let branch_action = self.insert_inspector(Node::Branch(children, None), partial, value, old_val)?.unwrap_node();
					InsertAction::Replace(branch_action)
				} else if cp == existing_key.len() {
					trace!(target: "trie", "complete-prefix (cp={:?}): AUGMENT-AT-END", cp);

					// fully-shared prefix.

					// insert into the child node.
					let (new_child, changed) = self.insert_at(child_branch, partial.mid(cp), value, old_val)?;
					let new_ext = Node::Extension(existing_key.encoded(false), new_child.into());

					// if the child branch wasn't changed, meaning this extension remains the same.
					match changed {
						true => InsertAction::Replace(new_ext),
						false => InsertAction::Restore(new_ext),
					}
				} else {
					trace!(target: "trie", "partially-shared-prefix (exist={:?}; new={:?}; cp={:?}): AUGMENT-AT-END", existing_key.len(), partial.len(), cp);

					// partially-shared.
					let low = Node::Extension(existing_key.mid(cp).encoded(false), child_branch);
					// augment the extension. this will take the cp == 0 path, creating a branch.
					let augmented_low = self.insert_inspector(low, partial.mid(cp), value, old_val)?.unwrap_node();

					// always replace, since this extension is not the one we started with.
					// this is known because the partial key is only the common prefix.
					InsertAction::Replace(Node::Extension(
						existing_key.encoded_leftmost(cp, false),
						self.storage.alloc(Stored::New(augmented_low)).into()
					))
				}
			}
		})
	}

	/// Remove a node from the trie based on key.
	fn remove_at(&mut self, handle: NodeHandle<H::Out>, partial: NibbleSlice, old_val: &mut Option<DBValue>) -> Result<Option<(StorageHandle, bool)>, H::Out, C::Error> {
		let stored = match handle {
			NodeHandle::InMemory(h) => self.storage.destroy(h),
			NodeHandle::Hash(h) => {
				let handle = self.cache(h)?;
				self.storage.destroy(handle)
			}
		};

		let opt = self.inspect(stored, move |trie, node| trie.remove_inspector(node, partial, old_val))?;

		Ok(opt.map(|(new, changed)| (self.storage.alloc(new), changed)))
	}

	/// the removal inspector
	fn remove_inspector(&mut self, node: Node<H::Out>, partial: NibbleSlice, old_val: &mut Option<DBValue>) -> Result<Action<H::Out>, H::Out, C::Error> {
		Ok(match (node, partial.is_empty()) {
			(Node::Empty, _) => Action::Delete,
			(Node::Branch(c, None), true) => Action::Restore(Node::Branch(c, None)),
			(Node::Branch(children, Some(val)), true) => {
				*old_val = Some(val);
				// always replace since we took the value out.
				Action::Replace(self.fix(Node::Branch(children, None))?)
			}
			(Node::Branch(mut children, value), false) => {
				let idx = partial.at(0) as usize;
				if let Some(child) = children[idx].take() {
					trace!(target: "trie", "removing value out of branch child, partial={:?}", partial);
					match self.remove_at(child, partial.mid(1), old_val)? {
						Some((new, changed)) => {
							children[idx] = Some(new.into());
							let branch = Node::Branch(children, value);
							match changed {
								// child was changed, so we were too.
								true => Action::Replace(branch),
								// unchanged, so we are too.
								false => Action::Restore(branch),
							}
						}
						None => {
							// the child we took was deleted.
							// the node may need fixing.
							trace!(target: "trie", "branch child deleted, partial={:?}", partial);
							Action::Replace(self.fix(Node::Branch(children, value))?)
						}
					}
				} else {
					// no change needed.
					Action::Restore(Node::Branch(children, value))
				}
			}
			(Node::Leaf(encoded, value), _) => {
				if NibbleSlice::from_encoded(&encoded).0 == partial {
					// this is the node we were looking for. Let's delete it.
					*old_val = Some(value);
					Action::Delete
				} else {
					// leaf the node alone.
					trace!(target: "trie", "restoring leaf wrong partial, partial={:?}, existing={:?}", partial, NibbleSlice::from_encoded(&encoded).0);
					Action::Restore(Node::Leaf(encoded, value))
				}
			}
			(Node::Extension(encoded, child_branch), _) => {
				let (cp, existing_len) = {
					let existing_key = NibbleSlice::from_encoded(&encoded).0;
					(existing_key.common_prefix(&partial), existing_key.len())
				};
				if cp == existing_len {
					// try to remove from the child branch.
					trace!(target: "trie", "removing from extension child, partial={:?}", partial);
					match self.remove_at(child_branch, partial.mid(cp), old_val)? {
						Some((new_child, changed)) => {
							let new_child = new_child.into();

							// if the child branch was unchanged, then the extension is too.
							// otherwise, this extension may need fixing.
							match changed {
								true => Action::Replace(self.fix(Node::Extension(encoded, new_child))?),
								false => Action::Restore(Node::Extension(encoded, new_child)),
							}
						}
						None => {
							// the whole branch got deleted.
							// that means that this extension is useless.
							Action::Delete
						}
					}
				} else {
					// partway through an extension -- nothing to do here.
					Action::Restore(Node::Extension(encoded, child_branch))
				}
			}
		})
	}

	/// Given a node which may be in an _invalid state_, fix it such that it is then in a valid
	/// state.
	///
	/// _invalid state_ means:
	/// - Branch node where there is only a single entry;
	/// - Extension node followed by anything other than a Branch node.
	fn fix(&mut self, node: Node<H::Out>) -> Result<Node<H::Out>, H::Out, C::Error> {
		match node {
			Node::Branch(mut children, value) => {
				// if only a single value, transmute to leaf/extension and feed through fixed.
				#[derive(Debug)]
				enum UsedIndex {
					None,
					One(u8),
					Many,
				};
				let mut used_index = UsedIndex::None;
				for i in 0..16 {
					match (children[i].is_none(), &used_index) {
						(false, &UsedIndex::None) => used_index = UsedIndex::One(i as u8),
						(false, &UsedIndex::One(_)) => {
							used_index = UsedIndex::Many;
							break;
						}
						_ => continue,
					}
				}

				match (used_index, value) {
					(UsedIndex::None, None) => panic!("Branch with no subvalues. Something went wrong."),
					(UsedIndex::One(a), None) => {
						// only one onward node. make an extension.
						let new_partial = NibbleSlice::new_offset(&[a], 1).encoded(false);
						let child = children[a as usize].take().expect("used_index only set if occupied; qed");
						let new_node = Node::Extension(new_partial, child);
						self.fix(new_node)
					}
					(UsedIndex::None, Some(value)) => {
						// make a leaf.
						trace!(target: "trie", "fixing: branch -> leaf");
						Ok(Node::Leaf(NibbleSlice::new(&[]).encoded(true), value))
					}
					(_, value) => {
						// all is well.
						trace!(target: "trie", "fixing: restoring branch");
						Ok(Node::Branch(children, value))
					}
				}
			}
			Node::Extension(partial, child) => {
				let stored = match child {
					NodeHandle::InMemory(h) => self.storage.destroy(h),
					NodeHandle::Hash(h) => {
						let handle = self.cache(h)?;
						self.storage.destroy(handle)
					}
				};

				let (child_node, maybe_hash) = match stored {
					Stored::New(node) => (node, None),
					Stored::Cached(node, hash) => (node, Some(hash))
				};

				match child_node {
					Node::Extension(sub_partial, sub_child) => {
						// combine with node below.
						if let Some(hash) = maybe_hash {
							// delete the cached child since we are going to replace it.
							self.death_row.insert(hash);
						}
						let partial = NibbleSlice::from_encoded(&partial).0;
						let sub_partial = NibbleSlice::from_encoded(&sub_partial).0;

						let new_partial = NibbleSlice::new_composed(&partial, &sub_partial);
						trace!(target: "trie", "fixing: extension combination. new_partial={:?}", new_partial);
						self.fix(Node::Extension(new_partial.encoded(false), sub_child))
					}
					Node::Leaf(sub_partial, value) => {
						// combine with node below.
						if let Some(hash) = maybe_hash {
							// delete the cached child since we are going to replace it.
							self.death_row.insert(hash);
						}
						let partial = NibbleSlice::from_encoded(&partial).0;
						let sub_partial = NibbleSlice::from_encoded(&sub_partial).0;

						let new_partial = NibbleSlice::new_composed(&partial, &sub_partial);
						trace!(target: "trie", "fixing: extension -> leaf. new_partial={:?}", new_partial);
						Ok(Node::Leaf(new_partial.encoded(true), value))
					}
					child_node => {
						trace!(target: "trie", "fixing: restoring extension");

						// reallocate the child node.
						let stored = if let Some(hash) = maybe_hash {
							Stored::Cached(child_node, hash)
						} else {
							Stored::New(child_node)
						};

						Ok(Node::Extension(partial, self.storage.alloc(stored).into()))
					}
				}
			}
			other => Ok(other), // only ext and branch need fixing.
		}
	}

	/// Commit the in-memory changes to disk, freeing their storage and
	/// updating the state root.
	pub fn commit(&mut self) {
		trace!(target: "trie", "Committing trie changes to db.");

		// always kill all the nodes on death row.
		trace!(target: "trie", "{:?} nodes to remove from db", self.death_row.len());
		for hash in self.death_row.drain() {
			self.db.remove(&hash);
		}

		let handle = match self.root_handle() {
			NodeHandle::Hash(_) => return, // no changes necessary.
			NodeHandle::InMemory(h) => h,
		};

		match self.storage.destroy(handle) {
			Stored::New(node) => {
				let encoded_root = node.into_encoded::<_, C, H>(|child| self.commit_child(child) );
				*self.root = self.db.insert(&encoded_root[..]);
				self.hash_count += 1;

				trace!(target: "trie", "encoded root node: {:?}", (&encoded_root[..]).pretty());
				self.root_handle = NodeHandle::Hash(*self.root);
			}
			Stored::Cached(node, hash) => {
				// probably won't happen, but update the root and move on.
				*self.root = hash;
				self.root_handle = NodeHandle::InMemory(self.storage.alloc(Stored::Cached(node, hash)));
			}
		}
	}

	/// Commit a node by hashing it and writing it to the db. Returns a
	/// `ChildReference` which in most cases carries a normal hash but for the
	/// case where we can fit the actual data in the `Hasher`s output type, we
	/// store the data inline. This function is used as the callback to the
	/// `into_encoded` method of `Node`.
	fn commit_child(&mut self, handle: NodeHandle<H::Out>) -> ChildReference<H::Out> {
		match handle {
			NodeHandle::Hash(hash) => ChildReference::Hash(hash),
			NodeHandle::InMemory(storage_handle) => {
				match self.storage.destroy(storage_handle) {
					Stored::Cached(_, hash) => ChildReference::Hash(hash),
					Stored::New(node) => {
						let encoded = node.into_encoded::<_, C, H>(|node_handle| self.commit_child(node_handle) );
						if encoded.len() >= H::LENGTH {
							let hash = self.db.insert(&encoded[..]);
							self.hash_count +=1;
							ChildReference::Hash(hash)
						} else {
							// it's a small value, so we cram it into a `H::Out` and tag with length
							let mut h = H::Out::default();
							let len = encoded.len();
							h.as_mut()[..len].copy_from_slice(&encoded[..len]);
							ChildReference::Inline(h, len)
						}
					}
				}
			}
		}
	}

	// a hack to get the root node's handle
	fn root_handle(&self) -> NodeHandle<H::Out> {
		match self.root_handle {
			NodeHandle::Hash(h) => NodeHandle::Hash(h),
			NodeHandle::InMemory(StorageHandle(x)) => NodeHandle::InMemory(StorageHandle(x)),
		}
	}
}

impl<'a, H, C> TrieMut<H, C> for TrieDBMut<'a, H, C>
where
	H: Hasher,
	C: NodeCodec<H>
{
	fn root(&mut self) -> &H::Out {
		self.commit();
		self.root
	}

	fn is_empty(&self) -> bool {
		match self.root_handle {
			NodeHandle::Hash(h) => h == C::hashed_null_node(),
			NodeHandle::InMemory(ref h) => match self.storage[h] {
				Node::Empty => true,
				_ => false,
			}
		}
	}

	fn get<'x, 'key>(&'x self, key: &'key [u8]) -> Result<Option<DBValue>, H::Out, C::Error>
		where 'x: 'key
	{
		self.lookup(NibbleSlice::new(key), &self.root_handle)
	}

	fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<Option<DBValue>, H::Out, C::Error> {
		if value.is_empty() { return self.remove(key) }

		let mut old_val = None;

		trace!(target: "trie", "insert: key={:?}, value={:?}", key.pretty(), value.pretty());

		let root_handle = self.root_handle();
		let (new_handle, changed) = self.insert_at(
			root_handle,
			NibbleSlice::new(key),
			DBValue::from_slice(value),
			&mut old_val,
		)?;

		trace!(target: "trie", "insert: altered trie={}", changed);
		self.root_handle = NodeHandle::InMemory(new_handle);

		Ok(old_val)
	}

	fn remove(&mut self, key: &[u8]) -> Result<Option<DBValue>, H::Out, C::Error> {
		trace!(target: "trie", "remove: key={:?}", key.pretty());

		let root_handle = self.root_handle();
		let key = NibbleSlice::new(key);
		let mut old_val = None;

		match self.remove_at(root_handle, key, &mut old_val)? {
			Some((handle, changed)) => {
				trace!(target: "trie", "remove: altered trie={}", changed);
				self.root_handle = NodeHandle::InMemory(handle);
			}
			None => {
				trace!(target: "trie", "remove: obliterated trie");
				self.root_handle = NodeHandle::Hash(C::hashed_null_node());
				*self.root = C::hashed_null_node();
			}
		}

		Ok(old_val)
	}
}

impl<'a, H, C> Drop for TrieDBMut<'a, H, C>
where
	H: Hasher,
	C: NodeCodec<H>
{
	fn drop(&mut self) {
		self.commit();
	}
}

#[cfg(test)]
mod tests {
	use bytes::ToPretty;
	use hashdb::{Hasher, HashDB};
	use keccak_hasher::KeccakHasher;
	use memorydb::MemoryDB;
	use rlp::{Decodable, Encodable};
	use triehash::trie_root;
	use patricia_trie_ethereum::RlpTrieStream;
	use standardmap::*;
	use ethtrie::{TrieDBMut, RlpCodec, trie::{TrieMut, NodeCodec}};
	use env_logger;
	use ethereum_types::H256;
	use DBValue;

	fn populate_trie<'db, H, C>(
		db: &'db mut HashDB<KeccakHasher, DBValue>,
		root: &'db mut H256,
		v: &[(Vec<u8>, Vec<u8>)]
	) -> TrieDBMut<'db>
	where
		H: Hasher,
		H::Out: Decodable + Encodable,
		C: NodeCodec<H>,
	{
		let mut t = TrieDBMut::new(db, root);
		for i in 0..v.len() {
			let key: &[u8]= &v[i].0;
			let val: &[u8] = &v[i].1;
			t.insert(key, val).unwrap();
		}
		t
	}

	fn unpopulate_trie<'db>(t: &mut TrieDBMut<'db>, v: &[(Vec<u8>, Vec<u8>)]) {
		for i in v {
			let key: &[u8]= &i.0;
			t.remove(key).unwrap();
		}
	}

	#[test]
	fn playpen() {
		env_logger::init();
		let mut seed = H256::new();
		for test_i in 0..10 {
			if test_i % 50 == 0 {
				debug!("{:?} of 10000 stress tests done", test_i);
			}
			let x = StandardMap {
				alphabet: Alphabet::Custom(b"@QWERTYUIOPASDFGHJKLZXCVBNM[/]^_".to_vec()),
				min_key: 5,
				journal_key: 0,
				value_mode: ValueMode::Index,
				count: 100,
			}.make_with(&mut seed);

			let real = trie_root::<KeccakHasher, RlpTrieStream,_, _, _>(x.clone());
			let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
			let mut root = H256::new();
			let mut memtrie = populate_trie::<_, RlpCodec>(&mut memdb, &mut root, &x);

			memtrie.commit();
			if *memtrie.root() != real {
				println!("TRIE MISMATCH");
				println!("");
				println!("{:?} vs {:?}", memtrie.root(), real);
				for i in &x {
					println!("{:?} -> {:?}", i.0.pretty(), i.1.pretty());
				}
			}
			assert_eq!(*memtrie.root(), real);
			unpopulate_trie(&mut memtrie, &x);
			memtrie.commit();
			if *memtrie.root() != RlpCodec::hashed_null_node() {
				println!("- TRIE MISMATCH");
				println!("");
				println!("{:?} vs {:?}", memtrie.root(), real);
				for i in &x {
					println!("{:?} -> {:?}", i.0.pretty(), i.1.pretty());
				}
			}
			assert_eq!(*memtrie.root(), RlpCodec::hashed_null_node());
		}
	}

	#[test]
	fn init() {
		let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		assert_eq!(*t.root(), RlpCodec::hashed_null_node());
	}

	#[test]
	fn insert_on_empty() {
		let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]).unwrap();
		assert_eq!(*t.root(), trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(vec![ (vec![0x01u8, 0x23], vec![0x01u8, 0x23]) ]));
	}

	#[test]
	fn remove_to_empty() {
		let big_value = b"00000000000000000000000000000000";

		let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
		let mut root = H256::new();
		let mut t1 = TrieDBMut::new(&mut memdb, &mut root);
		t1.insert(&[0x01, 0x23], big_value).unwrap();
		t1.insert(&[0x01, 0x34], big_value).unwrap();
		let mut memdb2 = MemoryDB::<KeccakHasher, DBValue>::new();
		let mut root2 = H256::new();
		let mut t2 = TrieDBMut::new(&mut memdb2, &mut root2);

		t2.insert(&[0x01], big_value).unwrap();
		t2.insert(&[0x01, 0x23], big_value).unwrap();
		t2.insert(&[0x01, 0x34], big_value).unwrap();
		t2.remove(&[0x01]).unwrap();
	}

	#[test]
	fn insert_replace_root() {
		let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]).unwrap();
		t.insert(&[0x01u8, 0x23], &[0x23u8, 0x45]).unwrap();
		assert_eq!(*t.root(), trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(vec![ (vec![0x01u8, 0x23], vec![0x23u8, 0x45]) ]));
	}

	#[test]
	fn insert_make_branch_root() {
		let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]).unwrap();
		t.insert(&[0x11u8, 0x23], &[0x11u8, 0x23]).unwrap();
		assert_eq!(*t.root(), trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(vec![
			(vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
			(vec![0x11u8, 0x23], vec![0x11u8, 0x23])
		]));
	}

	#[test]
	fn insert_into_branch_root() {
		let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]).unwrap();
		t.insert(&[0xf1u8, 0x23], &[0xf1u8, 0x23]).unwrap();
		t.insert(&[0x81u8, 0x23], &[0x81u8, 0x23]).unwrap();
		assert_eq!(*t.root(), trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(vec![
			(vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
			(vec![0x81u8, 0x23], vec![0x81u8, 0x23]),
			(vec![0xf1u8, 0x23], vec![0xf1u8, 0x23]),
		]));
	}

	#[test]
	fn insert_value_into_branch_root() {
		let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]).unwrap();
		t.insert(&[], &[0x0]).unwrap();
		assert_eq!(*t.root(), trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(vec![
			(vec![], vec![0x0]),
			(vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
		]));
	}

	#[test]
	fn insert_split_leaf() {
		let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]).unwrap();
		t.insert(&[0x01u8, 0x34], &[0x01u8, 0x34]).unwrap();
		assert_eq!(*t.root(), trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(vec![
			(vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
			(vec![0x01u8, 0x34], vec![0x01u8, 0x34]),
		]));
	}

	#[test]
	fn insert_split_extenstion() {
		let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01, 0x23, 0x45], &[0x01]).unwrap();
		t.insert(&[0x01, 0xf3, 0x45], &[0x02]).unwrap();
		t.insert(&[0x01, 0xf3, 0xf5], &[0x03]).unwrap();
		assert_eq!(*t.root(), trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(vec![
			(vec![0x01, 0x23, 0x45], vec![0x01]),
			(vec![0x01, 0xf3, 0x45], vec![0x02]),
			(vec![0x01, 0xf3, 0xf5], vec![0x03]),
		]));
	}

	#[test]
	fn insert_big_value() {
		let big_value0 = b"00000000000000000000000000000000";
		let big_value1 = b"11111111111111111111111111111111";

		let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], big_value0).unwrap();
		t.insert(&[0x11u8, 0x23], big_value1).unwrap();
		assert_eq!(*t.root(), trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(vec![
			(vec![0x01u8, 0x23], big_value0.to_vec()),
			(vec![0x11u8, 0x23], big_value1.to_vec())
		]));
	}

	#[test]
	fn insert_duplicate_value() {
		let big_value = b"00000000000000000000000000000000";

		let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], big_value).unwrap();
		t.insert(&[0x11u8, 0x23], big_value).unwrap();
		assert_eq!(*t.root(), trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(vec![
			(vec![0x01u8, 0x23], big_value.to_vec()),
			(vec![0x11u8, 0x23], big_value.to_vec())
		]));
	}

	#[test]
	fn test_at_empty() {
		let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
		let mut root = H256::new();
		let t = TrieDBMut::new(&mut memdb, &mut root);
		assert_eq!(t.get(&[0x5]).unwrap(), None);
	}

	#[test]
	fn test_at_one() {
		let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]).unwrap();
		assert_eq!(t.get(&[0x1, 0x23]).unwrap().unwrap(), DBValue::from_slice(&[0x1u8, 0x23]));
		t.commit();
		assert_eq!(t.get(&[0x1, 0x23]).unwrap().unwrap(), DBValue::from_slice(&[0x1u8, 0x23]));
	}

	#[test]
	fn test_at_three() {
		let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]).unwrap();
		t.insert(&[0xf1u8, 0x23], &[0xf1u8, 0x23]).unwrap();
		t.insert(&[0x81u8, 0x23], &[0x81u8, 0x23]).unwrap();
		assert_eq!(t.get(&[0x01, 0x23]).unwrap().unwrap(), DBValue::from_slice(&[0x01u8, 0x23]));
		assert_eq!(t.get(&[0xf1, 0x23]).unwrap().unwrap(), DBValue::from_slice(&[0xf1u8, 0x23]));
		assert_eq!(t.get(&[0x81, 0x23]).unwrap().unwrap(), DBValue::from_slice(&[0x81u8, 0x23]));
		assert_eq!(t.get(&[0x82, 0x23]).unwrap(), None);
		t.commit();
		assert_eq!(t.get(&[0x01, 0x23]).unwrap().unwrap(), DBValue::from_slice(&[0x01u8, 0x23]));
		assert_eq!(t.get(&[0xf1, 0x23]).unwrap().unwrap(), DBValue::from_slice(&[0xf1u8, 0x23]));
		assert_eq!(t.get(&[0x81, 0x23]).unwrap().unwrap(), DBValue::from_slice(&[0x81u8, 0x23]));
		assert_eq!(t.get(&[0x82, 0x23]).unwrap(), None);
	}

	#[test]
	fn stress() {
		let mut seed = H256::new();
		for _ in 0..50 {
			let x = StandardMap {
				alphabet: Alphabet::Custom(b"@QWERTYUIOPASDFGHJKLZXCVBNM[/]^_".to_vec()),
				min_key: 5,
				journal_key: 0,
				value_mode: ValueMode::Index,
				count: 4,
			}.make_with(&mut seed);

			let real = trie_root::<KeccakHasher, RlpTrieStream,_, _, _>(x.clone());
			let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
			let mut root = H256::new();
			let mut memtrie = populate_trie::<_, RlpCodec>(&mut memdb, &mut root, &x);
			let mut y = x.clone();
			y.sort_by(|ref a, ref b| a.0.cmp(&b.0));
			let mut memdb2 = MemoryDB::<KeccakHasher, DBValue>::new();
			let mut root2 = H256::new();
			let mut memtrie_sorted = populate_trie::<_, RlpCodec>(&mut memdb2, &mut root2, &y);
			if *memtrie.root() != real || *memtrie_sorted.root() != real {
				println!("TRIE MISMATCH");
				println!("");
				println!("ORIGINAL... {:?}", memtrie.root());
				for i in &x {
					println!("{:?} -> {:?}", i.0.pretty(), i.1.pretty());
				}
				println!("SORTED... {:?}", memtrie_sorted.root());
				for i in &y {
					println!("{:?} -> {:?}", i.0.pretty(), i.1.pretty());
				}
			}
			assert_eq!(*memtrie.root(), real);
			assert_eq!(*memtrie_sorted.root(), real);
		}
	}

	#[test]
	fn test_trie_existing() {
		let mut db = MemoryDB::<KeccakHasher, DBValue>::new();
		let mut root = H256::new();
		{
			let mut t = TrieDBMut::new(&mut db, &mut root);
			t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]).unwrap();
		}

		{
			 let _ = TrieDBMut::from_existing(&mut db, &mut root);
		}
	}

	#[test]
	fn insert_empty() {
		let mut seed = H256::new();
		let x = StandardMap {
				alphabet: Alphabet::Custom(b"@QWERTYUIOPASDFGHJKLZXCVBNM[/]^_".to_vec()),
				min_key: 5,
				journal_key: 0,
				value_mode: ValueMode::Index,
				count: 4,
		}.make_with(&mut seed);

		let mut db = MemoryDB::<KeccakHasher, DBValue>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut db, &mut root);
		for &(ref key, ref value) in &x {
			t.insert(key, value).unwrap();
		}

		assert_eq!(*t.root(), trie_root::<KeccakHasher, RlpTrieStream, _, _, _>(x.clone()));

		for &(ref key, _) in &x {
			t.insert(key, &[]).unwrap();
		}

		assert!(t.is_empty());
		assert_eq!(*t.root(), RlpCodec::hashed_null_node());
	}

	#[test]
	fn return_old_values() {
		let mut seed = H256::new();
		let x = StandardMap {
				alphabet: Alphabet::Custom(b"@QWERTYUIOPASDFGHJKLZXCVBNM[/]^_".to_vec()),
				min_key: 5,
				journal_key: 0,
				value_mode: ValueMode::Index,
				count: 4,
		}.make_with(&mut seed);

		let mut db = MemoryDB::<KeccakHasher, DBValue>::new();
		let mut root = H256::new();
		let mut t = TrieDBMut::new(&mut db, &mut root);
		for &(ref key, ref value) in &x {
			assert!(t.insert(key, value).unwrap().is_none());
			assert_eq!(t.insert(key, value).unwrap(), Some(DBValue::from_slice(value)));
		}

		for (key, value) in x {
			assert_eq!(t.remove(&key).unwrap(), Some(DBValue::from_slice(&value)));
			assert!(t.remove(&key).unwrap().is_none());
		}
	}
}
