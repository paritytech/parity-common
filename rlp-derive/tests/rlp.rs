// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use rlp::{decode, encode};
use rlp_derive::{RlpDecodable, RlpDecodableWrapper, RlpEncodable, RlpEncodableWrapper};

#[derive(Debug, PartialEq, RlpEncodable, RlpDecodable)]
struct Item {
	a: String,
}

#[derive(Debug, PartialEq, RlpEncodableWrapper, RlpDecodableWrapper)]
struct ItemWrapper {
	a: String,
}

#[test]
fn test_encode_item() {
	let item = Item { a: "cat".into() };

	let expected = vec![0xc4, 0x83, b'c', b'a', b't'];
	let out = encode(&item);
	assert_eq!(out, expected);

	let decoded = decode(&expected).expect("decode failure");
	assert_eq!(item, decoded);
}

#[test]
fn test_encode_item_wrapper() {
	let item = ItemWrapper { a: "cat".into() };

	let expected = vec![0x83, b'c', b'a', b't'];
	let out = encode(&item);
	assert_eq!(out, expected);

	let decoded = decode(&expected).expect("decode failure");
	assert_eq!(item, decoded);
}

#[test]
fn test_encode_item_default() {
	#[derive(Debug, PartialEq, RlpEncodable, RlpDecodable)]
	struct ItemDefault {
		a: String,
		/// It works with other attributes.
		#[rlp(default)]
		b: Option<Vec<u8>>,
	}

	let attack_of = "clones";
	let item = Item { a: attack_of.into() };

	let expected = vec![0xc7, 0x86, b'c', b'l', b'o', b'n', b'e', b's'];
	let out = encode(&item);
	assert_eq!(out, expected);

	let item_default = ItemDefault { a: attack_of.into(), b: None };

	let decoded = decode(&expected).expect("default failure");
	assert_eq!(item_default, decoded);

	let item_some = ItemDefault { a: attack_of.into(), b: Some(vec![1, 2, 3]) };
	let out = encode(&item_some);
	assert_eq!(decode(&out), Ok(item_some));
}
