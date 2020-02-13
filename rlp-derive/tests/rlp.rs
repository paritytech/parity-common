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
struct Foo {
	a: String,
}

#[derive(Debug, PartialEq, RlpEncodableWrapper, RlpDecodableWrapper)]
struct FooWrapper {
	a: String,
}

#[test]
fn test_encode_foo() {
	let foo = Foo { a: "cat".into() };

	let expected = vec![0xc4, 0x83, b'c', b'a', b't'];
	let out = encode(&foo);
	assert_eq!(out, expected);

	let decoded = decode(&expected).expect("decode failure");
	assert_eq!(foo, decoded);
}

#[test]
fn test_encode_foo_wrapper() {
	let foo = FooWrapper { a: "cat".into() };

	let expected = vec![0x83, b'c', b'a', b't'];
	let out = encode(&foo);
	assert_eq!(out, expected);

	let decoded = decode(&expected).expect("decode failure");
	assert_eq!(foo, decoded);
}

#[test]
fn test_encode_foo_default() {
	#[derive(Debug, PartialEq, RlpEncodable, RlpDecodable)]
	struct FooDefault {
		a: String,
		/// It works with other attributes.
		#[rlp(default)]
		b: Option<Vec<u8>>,
	}

	let attack_of = String::from("clones");
	let foo = Foo { a: attack_of.clone() };

	let expected = vec![0xc7, 0x86, b'c', b'l', b'o', b'n', b'e', b's'];
	let out = encode(&foo);
	assert_eq!(out, expected);

	let foo_default = FooDefault { a: attack_of.clone(), b: None };

	let decoded = decode(&expected).expect("default failure");
	assert_eq!(foo_default, decoded);

	let foo_some = FooDefault { a: attack_of.clone(), b: Some(vec![1, 2, 3]) };
	let out = encode(&foo_some);
	assert_eq!(decode(&out), Ok(foo_some));
}
