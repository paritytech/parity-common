use hashdb::Hasher;
use ethereum_types::H256;
use plain_hasher::PlainHasher;
use rlp::{DecoderError, RlpStream, Rlp, Prototype};
use super::{NibbleSlice, node::Node, ChildReference, NodeCodec};
use std::marker::PhantomData;
use elastic_array::{ElasticArray1024, ElasticArray128};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TestHasher;
impl Hasher for TestHasher {
	type Out = H256;
	type StdHasher = PlainHasher;
	const LENGTH: usize = 32;
	fn hash(x: &[u8]) -> Self::Out {
		let mut out = [0;32];
		::tiny_keccak::Keccak::keccak256(x, &mut out);
		out.into()
	}
}

#[derive(Default, Clone)]
pub struct RlpNodeCodec<H: Hasher> {mark: PhantomData<H>}

impl NodeCodec<TestHasher> for RlpNodeCodec<TestHasher> {
	type Error = DecoderError;
	const HASHED_NULL_NODE : H256 = H256( [0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e, 0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21] );
	fn decode(data: &[u8]) -> ::std::result::Result<Node, Self::Error> {
		let r = Rlp::new(data);
		match r.prototype()? {
			// either leaf or extension - decode first item with NibbleSlice::???
			// and use is_leaf return to figure out which.
			// if leaf, second item is a value (is_data())
			// if extension, second item is a node (either SHA3 to be looked up and
			// fed back into this function or inline RLP which can be fed back into this function).
			Prototype::List(2) => match NibbleSlice::from_encoded(r.at(0)?.data()?) {
				(slice, true) => Ok(Node::Leaf(slice, r.at(1)?.data()?)),
				(slice, false) => Ok(Node::Extension(slice, r.at(1)?.as_raw())),
			},
			// branch - first 16 are nodes, 17th is a value (or empty).
			Prototype::List(17) => {
				let mut nodes = [&[] as &[u8]; 16];
				for i in 0..16 {
					nodes[i] = r.at(i)?.as_raw();
				}
				Ok(Node::Branch(nodes, if r.at(16)?.is_empty() { None } else { Some(r.at(16)?.data()?) }))
			},
			// an empty branch index.
			Prototype::Data(0) => Ok(Node::Empty),
			// something went wrong.
			_ => Err(DecoderError::Custom("Rlp is not valid."))
		}
	}
	fn try_decode_hash(data: &[u8]) -> Option<<TestHasher as Hasher>::Out> {
		let r = Rlp::new(data);
		if r.is_data() && r.size() == TestHasher::LENGTH {
			Some(r.as_val().expect("Hash is the correct size; qed"))
		} else {
			None
		}
	}
	fn is_empty_node(data: &[u8]) -> bool {
		Rlp::new(data).is_empty()
	}
	fn empty_node() -> ElasticArray1024<u8> {
		let mut stream = RlpStream::new();
		stream.append_empty_data();
		stream.drain()
	}

	fn leaf_node(partial: &[u8], value: &[u8]) -> ElasticArray1024<u8> {
		let mut stream = RlpStream::new_list(2);
		stream.append(&partial);
		stream.append(&value);
		stream.drain()
	}

	fn ext_node(partial: &[u8], child_ref: ChildReference<<TestHasher as Hasher>::Out>) -> ElasticArray1024<u8> {
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

	fn branch_node<I>(children: I, value: Option<ElasticArray128<u8>>) -> ElasticArray1024<u8>
	where I: IntoIterator<Item=Option<ChildReference<<TestHasher as Hasher>::Out>>>
	{
		let mut stream = RlpStream::new_list(17);
		for child_ref in children {
			match child_ref {
				Some(c) => match c {
					ChildReference::Hash(h) => stream.append(&h),
					ChildReference::Inline(inline_data, len) => {
						let bytes = &AsRef::<[u8]>::as_ref(&inline_data)[..len];
						stream.append_raw(bytes, 1)
					},
				},
				None => stream.append_empty_data()
			};
		}
		if let Some(value) = value {
			stream.append(&&*value);
		} else {
			stream.append_empty_data();
		}
		stream.drain()
	}
}

pub type RlpCodec = RlpNodeCodec<TestHasher>;
pub type TrieDB<'db> = super::TrieDB<'db, TestHasher, RlpCodec>;
pub type SecTrieDB<'db> = super::SecTrieDB<'db, TestHasher, RlpCodec>;
pub type FatDB<'db> = super::FatDB<'db, TestHasher, RlpCodec>;
pub type TrieDBMut<'db> = super::TrieDBMut<'db, TestHasher, RlpCodec>;
pub type SecTrieDBMut<'db> = super::SecTrieDBMut<'db, TestHasher, RlpCodec>;
pub type FatDBMut<'db> = super::FatDBMut<'db, TestHasher, RlpCodec>;