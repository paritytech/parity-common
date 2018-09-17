use hex_prefix_encoding::hex_prefix_encode;
use rlp::{RlpStream, encode as rlp_encode};
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
	fn append_leaf<H: Hasher>(&mut self, key: &[u8], value: &[u8]) {
		self.stream.begin_list(2);
		self.stream.append_iter(hex_prefix_encode(key, true));
		self.stream.append(&value);
	}

	fn out(self) -> Vec<u8> { self.stream.out() }

	fn as_raw(&self) -> &[u8] { &self.stream.as_raw() }

	fn encode(k: &usize) -> Vec<u8> {
		rlp_encode(k)
	}
}
