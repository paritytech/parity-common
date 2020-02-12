// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::{cmp, fmt};

use hex_literal::hex;
use primitive_types::{H160, U256};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[test]
fn test_rlp_display() {
	let data = hex!("f84d0589010efbef67941f79b2a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a0c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470");
	let rlp = Rlp::new(&data);
	assert_eq!(format!("{}", rlp), "[\"0x05\", \"0x010efbef67941f79b2\", \"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421\", \"0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470\"]");
}

#[test]
fn length_overflow() {
	let bs = [0xbf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xe5];
	let rlp = Rlp::new(&bs);
	let res: Result<u8, DecoderError> = rlp.as_val();
	assert_eq!(Err(DecoderError::RlpInvalidLength), res);
}

#[test]
fn rlp_at() {
	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	{
		let rlp = Rlp::new(&data);
		assert!(rlp.is_list());
		let animals: Vec<String> = rlp.as_list().unwrap();
		assert_eq!(animals, vec!["cat".to_owned(), "dog".to_owned()]);

		let cat = rlp.at(0).unwrap();
		assert!(cat.is_data());
		assert_eq!(cat.as_raw(), &[0x83, b'c', b'a', b't']);
		assert_eq!(cat.as_val::<String>().unwrap(), "cat".to_owned());

		let dog = rlp.at(1).unwrap();
		assert!(dog.is_data());
		assert_eq!(dog.as_raw(), &[0x83, b'd', b'o', b'g']);
		assert_eq!(dog.as_val::<String>().unwrap(), "dog".to_owned());

		let cat_again = rlp.at(0).unwrap();
		assert!(cat_again.is_data());
		assert_eq!(cat_again.as_raw(), &[0x83, b'c', b'a', b't']);
		assert_eq!(cat_again.as_val::<String>().unwrap(), "cat".to_owned());
	}
}

#[test]
fn rlp_at_with_offset() {
	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	{
		let rlp = Rlp::new(&data);
		assert!(rlp.is_list());
		let animals: Vec<String> = rlp.as_list().unwrap();
		assert_eq!(animals, vec!["cat".to_owned(), "dog".to_owned()]);

		let (cat, cat_offset) = rlp.at_with_offset(0).unwrap();
		assert!(cat.is_data());
		assert_eq!(cat_offset, 1);
		assert_eq!(cat.as_raw(), &[0x83, b'c', b'a', b't']);
		assert_eq!(cat.as_val::<String>().unwrap(), "cat".to_owned());

		let (dog, dog_offset) = rlp.at_with_offset(1).unwrap();
		assert!(dog.is_data());
		assert_eq!(dog_offset, 5);
		assert_eq!(dog.as_raw(), &[0x83, b'd', b'o', b'g']);
		assert_eq!(dog.as_val::<String>().unwrap(), "dog".to_owned());

		let (cat_again, cat_offset) = rlp.at_with_offset(0).unwrap();
		assert!(cat_again.is_data());
		assert_eq!(cat_offset, 1);
		assert_eq!(cat_again.as_raw(), &[0x83, b'c', b'a', b't']);
		assert_eq!(cat_again.as_val::<String>().unwrap(), "cat".to_owned());
	}
}

#[test]
fn rlp_at_err() {
	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o'];
	{
		let rlp = Rlp::new(&data);
		assert!(rlp.is_list());

		let cat_err = rlp.at(0).unwrap_err();
		assert_eq!(cat_err, DecoderError::RlpIsTooShort);

		let dog_err = rlp.at(1).unwrap_err();
		assert_eq!(dog_err, DecoderError::RlpIsTooShort);
	}
}

#[test]
fn rlp_iter() {
	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	{
		let rlp = Rlp::new(&data);
		let mut iter = rlp.iter();

		let cat = iter.next().unwrap();
		assert!(cat.is_data());
		assert_eq!(cat.as_raw(), &[0x83, b'c', b'a', b't']);

		let dog = iter.next().unwrap();
		assert!(dog.is_data());
		assert_eq!(dog.as_raw(), &[0x83, b'd', b'o', b'g']);

		let none = iter.next();
		assert!(none.is_none());

		let cat_again = rlp.at(0).unwrap();
		assert!(cat_again.is_data());
		assert_eq!(cat_again.as_raw(), &[0x83, b'c', b'a', b't']);
	}
}

struct ETestPair<T>(T, Vec<u8>)
where
	T: Encodable;

fn run_encode_tests<T>(tests: Vec<ETestPair<T>>)
where
	T: Encodable,
{
	for t in &tests {
		let res = rlp::encode(&t.0);
		assert_eq!(&res[..], &t.1[..]);
	}
}

struct VETestPair<T>(Vec<T>, Vec<u8>)
where
	T: Encodable;

fn run_encode_tests_list<T>(tests: Vec<VETestPair<T>>)
where
	T: Encodable,
{
	for t in &tests {
		let res = rlp::encode_list(&t.0);
		assert_eq!(&res[..], &t.1[..]);
	}
}

#[test]
fn encode_u16() {
	let tests = vec![
		ETestPair(0u16, vec![0x80u8]),
		ETestPair(0x100, vec![0x82, 0x01, 0x00]),
		ETestPair(0xffff, vec![0x82, 0xff, 0xff]),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_u32() {
	let tests = vec![
		ETestPair(0u32, vec![0x80u8]),
		ETestPair(0x0001_0000, vec![0x83, 0x01, 0x00, 0x00]),
		ETestPair(0x00ff_ffff, vec![0x83, 0xff, 0xff, 0xff]),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_u64() {
	let tests = vec![
		ETestPair(0u64, vec![0x80u8]),
		ETestPair(0x0100_0000, vec![0x84, 0x01, 0x00, 0x00, 0x00]),
		ETestPair(0xFFFF_FFFF, vec![0x84, 0xff, 0xff, 0xff, 0xff]),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_u256() {
	let tests = vec![
		ETestPair(U256::from(0u64), vec![0x80u8]),
		ETestPair(U256::from(0x0100_0000u64), vec![0x84, 0x01, 0x00, 0x00, 0x00]),
		ETestPair(U256::from(0xffff_ffffu64), vec![0x84, 0xff, 0xff, 0xff, 0xff]),
		ETestPair(
			("8090a0b0c0d0e0f00910203040506077000000000000\
			  000100000000000012f0")
				.into(),
			vec![
				0xa0, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0, 0x09, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x77,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0xf0,
			],
		),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_str() {
	let tests = vec![
		ETestPair("cat", vec![0x83, b'c', b'a', b't']),
		ETestPair("dog", vec![0x83, b'd', b'o', b'g']),
		ETestPair("Marek", vec![0x85, b'M', b'a', b'r', b'e', b'k']),
		ETestPair("", vec![0x80]),
		ETestPair(
			"Lorem ipsum dolor sit amet, consectetur adipisicing elit",
			vec![
				0xb8, 0x38, b'L', b'o', b'r', b'e', b'm', b' ', b'i', b'p', b's', b'u', b'm', b' ', b'd', b'o', b'l',
				b'o', b'r', b' ', b's', b'i', b't', b' ', b'a', b'm', b'e', b't', b',', b' ', b'c', b'o', b'n', b's',
				b'e', b'c', b't', b'e', b't', b'u', b'r', b' ', b'a', b'd', b'i', b'p', b'i', b's', b'i', b'c', b'i',
				b'n', b'g', b' ', b'e', b'l', b'i', b't',
			],
		),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_address() {
	let tests = vec![ETestPair(
		H160::from(hex!("ef2d6d194084c2de36e0dabfce45d046b37d1106")),
		vec![
			0x94, 0xef, 0x2d, 0x6d, 0x19, 0x40, 0x84, 0xc2, 0xde, 0x36, 0xe0, 0xda, 0xbf, 0xce, 0x45, 0xd0, 0x46, 0xb3,
			0x7d, 0x11, 0x06,
		],
	)];
	run_encode_tests(tests);
}

/// Vec<u8> (Bytes) is treated as a single value
#[test]
fn encode_vector_u8() {
	let tests = vec![
		ETestPair(vec![], vec![0x80]),
		ETestPair(vec![0u8], vec![0]),
		ETestPair(vec![0x15], vec![0x15]),
		ETestPair(vec![0x40, 0x00], vec![0x82, 0x40, 0x00]),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_vector_u64() {
	let tests = vec![
		VETestPair(vec![], vec![0xc0]),
		VETestPair(vec![15u64], vec![0xc1, 0x0f]),
		VETestPair(vec![1, 2, 3, 7, 0xff], vec![0xc6, 1, 2, 3, 7, 0x81, 0xff]),
		VETestPair(
			vec![0xffff_ffff, 1, 2, 3, 7, 0xff],
			vec![0xcb, 0x84, 0xff, 0xff, 0xff, 0xff, 1, 2, 3, 7, 0x81, 0xff],
		),
	];
	run_encode_tests_list(tests);
}

#[test]
fn encode_vector_str() {
	let tests = vec![VETestPair(vec!["cat", "dog"], vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'])];
	run_encode_tests_list(tests);
}

struct DTestPair<T>(T, Vec<u8>)
where
	T: Decodable + fmt::Debug + cmp::Eq;

struct VDTestPair<T>(Vec<T>, Vec<u8>)
where
	T: Decodable + fmt::Debug + cmp::Eq;

fn run_decode_tests<T>(tests: Vec<DTestPair<T>>)
where
	T: Decodable + fmt::Debug + cmp::Eq,
{
	for t in &tests {
		let res: Result<T, DecoderError> = rlp::decode(&t.1);
		assert!(res.is_ok());
		let res = res.unwrap();
		assert_eq!(&res, &t.0);
	}
}

fn run_decode_tests_list<T>(tests: Vec<VDTestPair<T>>)
where
	T: Decodable + fmt::Debug + cmp::Eq,
{
	for t in &tests {
		let res: Vec<T> = rlp::decode_list(&t.1);
		assert_eq!(res, t.0);
	}
}

/// Vec<u8> (Bytes) is treated as a single value
#[test]
fn decode_vector_u8() {
	let tests = vec![
		DTestPair(vec![], vec![0x80]),
		DTestPair(vec![0u8], vec![0]),
		DTestPair(vec![0x15], vec![0x15]),
		DTestPair(vec![0x40, 0x00], vec![0x82, 0x40, 0x00]),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_u8() {
	let tests = vec![DTestPair(0x0u8, vec![0x80]), DTestPair(0x77u8, vec![0x77]), DTestPair(0xccu8, vec![0x81, 0xcc])];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_u16() {
	let tests = vec![DTestPair(0x100u16, vec![0x82, 0x01, 0x00]), DTestPair(0xffffu16, vec![0x82, 0xff, 0xff])];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_u32() {
	let tests = vec![
		DTestPair(0x0001_0000u32, vec![0x83, 0x01, 0x00, 0x00]),
		DTestPair(0x00ff_ffffu32, vec![0x83, 0xff, 0xff, 0xff]),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_u64() {
	let tests = vec![
		DTestPair(0x0100_0000u64, vec![0x84, 0x01, 0x00, 0x00, 0x00]),
		DTestPair(0xFFFF_FFFFu64, vec![0x84, 0xff, 0xff, 0xff, 0xff]),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_u256() {
	let tests = vec![
		DTestPair(U256::from(0u64), vec![0x80u8]),
		DTestPair(U256::from(0x0100_0000u64), vec![0x84, 0x01, 0x00, 0x00, 0x00]),
		DTestPair(U256::from(0xffff_ffffu64), vec![0x84, 0xff, 0xff, 0xff, 0xff]),
		DTestPair(
			("8090a0b0c0d0e0f00910203040506077000000000000\
			  000100000000000012f0")
				.into(),
			vec![
				0xa0, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0, 0x09, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x77,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0xf0,
			],
		),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_str() {
	let tests = vec![
		DTestPair("cat".to_owned(), vec![0x83, b'c', b'a', b't']),
		DTestPair("dog".to_owned(), vec![0x83, b'd', b'o', b'g']),
		DTestPair("Marek".to_owned(), vec![0x85, b'M', b'a', b'r', b'e', b'k']),
		DTestPair("".to_owned(), vec![0x80]),
		DTestPair(
			"Lorem ipsum dolor sit amet, consectetur adipisicing elit".to_owned(),
			vec![
				0xb8, 0x38, b'L', b'o', b'r', b'e', b'm', b' ', b'i', b'p', b's', b'u', b'm', b' ', b'd', b'o', b'l',
				b'o', b'r', b' ', b's', b'i', b't', b' ', b'a', b'm', b'e', b't', b',', b' ', b'c', b'o', b'n', b's',
				b'e', b'c', b't', b'e', b't', b'u', b'r', b' ', b'a', b'd', b'i', b'p', b'i', b's', b'i', b'c', b'i',
				b'n', b'g', b' ', b'e', b'l', b'i', b't',
			],
		),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_address() {
	let tests = vec![DTestPair(
		H160::from(hex!("ef2d6d194084c2de36e0dabfce45d046b37d1106")),
		vec![
			0x94, 0xef, 0x2d, 0x6d, 0x19, 0x40, 0x84, 0xc2, 0xde, 0x36, 0xe0, 0xda, 0xbf, 0xce, 0x45, 0xd0, 0x46, 0xb3,
			0x7d, 0x11, 0x06,
		],
	)];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_vector_u64() {
	let tests = vec![
		VDTestPair(vec![], vec![0xc0]),
		VDTestPair(vec![15u64], vec![0xc1, 0x0f]),
		VDTestPair(vec![1, 2, 3, 7, 0xff], vec![0xc6, 1, 2, 3, 7, 0x81, 0xff]),
		VDTestPair(
			vec![0xffff_ffff, 1, 2, 3, 7, 0xff],
			vec![0xcb, 0x84, 0xff, 0xff, 0xff, 0xff, 1, 2, 3, 7, 0x81, 0xff],
		),
	];
	run_decode_tests_list(tests);
}

#[test]
fn decode_untrusted_vector_str() {
	let tests = vec![VDTestPair(
		vec!["cat".to_owned(), "dog".to_owned()],
		vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'],
	)];
	run_decode_tests_list(tests);
}

#[test]
fn test_rlp_data_length_check() {
	let data = vec![0x84, b'c', b'a', b't'];
	let rlp = Rlp::new(&data);

	let as_val: Result<String, DecoderError> = rlp.as_val();
	assert_eq!(Err(DecoderError::RlpInconsistentLengthAndData), as_val);
}

#[test]
fn test_rlp_long_data_length_check() {
	let mut data: Vec<u8> = vec![0xb8, 255];
	for _ in 0..253 {
		data.push(b'c');
	}

	let rlp = Rlp::new(&data);

	let as_val: Result<String, DecoderError> = rlp.as_val();
	assert_eq!(Err(DecoderError::RlpInconsistentLengthAndData), as_val);
}

#[test]
fn test_the_exact_long_string() {
	let mut data: Vec<u8> = vec![0xb8, 255];
	for _ in 0..255 {
		data.push(b'c');
	}

	let rlp = Rlp::new(&data);

	let as_val: Result<String, DecoderError> = rlp.as_val();
	assert!(as_val.is_ok());
}

#[test]
fn test_rlp_2bytes_data_length_check() {
	let mut data: Vec<u8> = vec![0xb9, 2, 255]; // 512+255
	for _ in 0..700 {
		data.push(b'c');
	}

	let rlp = Rlp::new(&data);

	let as_val: Result<String, DecoderError> = rlp.as_val();
	assert_eq!(Err(DecoderError::RlpInconsistentLengthAndData), as_val);
}

#[test]
fn test_rlp_nested_empty_list_encode() {
	let mut stream = RlpStream::new_list(2);
	stream.append_list(&(Vec::new() as Vec<u32>));
	stream.append(&40u32);
	assert_eq!(stream.drain()[..], [0xc2u8, 0xc0u8, 40u8][..]);
}

#[test]
fn test_rlp_list_length_overflow() {
	let data: Vec<u8> = vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00];
	let rlp = Rlp::new(&data);
	let as_val: Result<String, DecoderError> = rlp.val_at(0);
	assert_eq!(Err(DecoderError::RlpIsTooShort), as_val);
}

#[test]
fn test_rlp_stream_size_limit() {
	for limit in 40..270 {
		let item = [0u8; 1];
		let mut stream = RlpStream::new();
		while stream.append_raw_checked(&item, 1, limit) {}
		assert_eq!(stream.drain().len(), limit);
	}
}

#[test]
fn test_rlp_stream_unbounded_list() {
	let mut stream = RlpStream::new();
	stream.begin_unbounded_list();
	stream.append(&40u32);
	stream.append(&41u32);
	assert!(!stream.is_finished());
	stream.finalize_unbounded_list();
	assert!(stream.is_finished());
}

#[test]
fn test_rlp_is_int() {
	for b in 0xb8..0xc0 {
		let data: Vec<u8> = vec![b];
		let rlp = Rlp::new(&data);
		assert_eq!(rlp.is_int(), false);
	}
}

// test described in
//
// https://github.com/paritytech/parity-common/issues/49
#[test]
fn test_canonical_string_encoding() {
	assert_ne!(
		Rlp::new(&[0xc0 + 4, 0xb7 + 1, 2, b'a', b'b']).val_at::<String>(0),
		Rlp::new(&[0xc0 + 3, 0x82, b'a', b'b']).val_at::<String>(0)
	);

	assert_eq!(
		Rlp::new(&[0xc0 + 4, 0xb7 + 1, 2, b'a', b'b']).val_at::<String>(0),
		Err(DecoderError::RlpInvalidIndirection)
	);
}

// test described in
//
// https://github.com/paritytech/parity-common/issues/49
#[test]
fn test_canonical_list_encoding() {
	assert_ne!(
		Rlp::new(&[0xc0 + 3, 0x82, b'a', b'b']).val_at::<String>(0),
		Rlp::new(&[0xf7 + 1, 3, 0x82, b'a', b'b']).val_at::<String>(0)
	);

	assert_eq!(
		Rlp::new(&[0xf7 + 1, 3, 0x82, b'a', b'b']).val_at::<String>(0),
		Err(DecoderError::RlpInvalidIndirection)
	);
}

// test described in
//
// https://github.com/paritytech/parity-common/issues/48
#[test]
fn test_inner_length_capping_for_short_lists() {
	assert_eq!(Rlp::new(&[0xc0, 0x82, b'a', b'b']).val_at::<String>(0), Err(DecoderError::RlpIsTooShort));
	assert_eq!(Rlp::new(&[0xc0 + 1, 0x82, b'a', b'b']).val_at::<String>(0), Err(DecoderError::RlpIsTooShort));
	assert_eq!(Rlp::new(&[0xc0 + 2, 0x82, b'a', b'b']).val_at::<String>(0), Err(DecoderError::RlpIsTooShort));
	assert_eq!(Rlp::new(&[0xc0 + 3, 0x82, b'a', b'b']).val_at::<String>(0), Ok("ab".to_owned()));
	assert_eq!(Rlp::new(&[0xc0 + 4, 0x82, b'a', b'b']).val_at::<String>(0), Err(DecoderError::RlpIsTooShort));
}

// test described in
//
// https://github.com/paritytech/parity-common/issues/105
#[test]
fn test_nested_list_roundtrip() {
	#[derive(Clone, Copy, Debug, PartialEq, Eq)]
	struct Inner(u64, u64);

	impl Encodable for Inner {
		fn rlp_append(&self, s: &mut RlpStream) {
			s.begin_unbounded_list().append(&self.0).append(&self.1).finalize_unbounded_list();
		}
	}

	impl Decodable for Inner {
		fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
			Ok(Inner(rlp.val_at(0)?, rlp.val_at(1)?))
		}
	}

	#[derive(Debug, Clone, PartialEq, Eq)]
	struct Nest<T>(Vec<T>);

	impl<T: Encodable> Encodable for Nest<T> {
		fn rlp_append(&self, s: &mut RlpStream) {
			s.begin_unbounded_list().append_list(&self.0).finalize_unbounded_list();
		}
	}

	impl<T: Decodable> Decodable for Nest<T> {
		fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
			Ok(Nest(rlp.list_at(0)?))
		}
	}

	let items = (0..4).map(|i| Inner(i, i + 1)).collect();
	let nest = Nest(items);

	let encoded = rlp::encode(&nest);
	let decoded = rlp::decode(&encoded).unwrap();

	assert_eq!(nest, decoded);

	let nest2 = Nest(vec![nest.clone(), nest]);

	let encoded = rlp::encode(&nest2);
	let decoded = rlp::decode(&encoded).unwrap();

	assert_eq!(nest2, decoded);
}

// test described in
//
// https://github.com/paritytech/parity-ethereum/pull/9663
#[test]
fn test_list_at() {
	let raw = hex!("f83e82022bd79020010db83c4d001500000000abcdef12820cfa8215a8d79020010db885a308d313198a2e037073488208ae82823a8443b9a355c5010203040531b9019afde696e582a78fa8d95ea13ce3297d4afb8ba6433e4154caa5ac6431af1b80ba76023fa4090c408f6b4bc3701562c031041d4702971d102c9ab7fa5eed4cd6bab8f7af956f7d565ee1917084a95398b6a21eac920fe3dd1345ec0a7ef39367ee69ddf092cbfe5b93e5e568ebc491983c09c76d922dc3");

	let rlp = Rlp::new(&raw);
	let _rlp1 = rlp.at(1).unwrap();
	let rlp2 = rlp.at(2).unwrap();
	assert_eq!(rlp2.val_at::<u16>(2).unwrap(), 33338);
}
