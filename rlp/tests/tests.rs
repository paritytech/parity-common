// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::{cmp, fmt};

use bytes::{Bytes, BytesMut};
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
	let bs = hex!("bfffffffffffffffffffffffe5");
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

impl<T, Repr> From<(T, Repr)> for ETestPair<T>
where
	T: Encodable,
	Repr: Into<Vec<u8>>,
{
	fn from((v, repr): (T, Repr)) -> Self {
		Self(v, repr.into())
	}
}

impl<T, Repr> From<(Vec<T>, Repr)> for VETestPair<T>
where
	T: Encodable,
	Repr: Into<Vec<u8>>,
{
	fn from((v, repr): (Vec<T>, Repr)) -> Self {
		Self(v, repr.into())
	}
}

#[test]
fn encode_u16() {
	let tests = vec![
		ETestPair::from((0_u16, hex!("80"))),
		ETestPair::from((0x100_u16, hex!("820100"))),
		ETestPair::from((0xffff_u16, hex!("82ffff"))),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_u32() {
	let tests = vec![
		ETestPair::from((0_u32, hex!("80"))),
		ETestPair::from((0x0001_0000_u32, hex!("83010000"))),
		ETestPair::from((0x00ff_ffff_u32, hex!("83ffffff"))),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_u64() {
	let tests = vec![
		ETestPair::from((0_u64, hex!("80"))),
		ETestPair::from((0x0100_0000_u64, hex!("8401000000"))),
		ETestPair::from((0xFFFF_FFFF_u64, hex!("84ffffffff"))),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_u128() {
	let tests = vec![
		ETestPair::from((0_u128, hex!("80"))),
		ETestPair::from((0x0100_0000_0000_0000_u128, hex!("880100000000000000"))),
		ETestPair::from((0xFFFF_FFFF_FFFF_FFFF_u128, hex!("88ffffffffffffffff"))),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_u256() {
	let tests = vec![
		ETestPair::from((U256::from(0_u64), hex!("80"))),
		ETestPair::from((U256::from(0x0100_0000_u64), hex!("8401000000"))),
		ETestPair::from((U256::from(0xffff_ffff_u64), hex!("84ffffffff"))),
		ETestPair::from((
			hex!("  8090a0b0c0d0e0f00910203040506077000000000000000100000000000012f0").into(),
			hex!("a08090a0b0c0d0e0f00910203040506077000000000000000100000000000012f0"),
		)),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_str() {
	let tests = vec![
		ETestPair::from(("cat", vec![0x83, b'c', b'a', b't'])),
		ETestPair::from(("dog", vec![0x83, b'd', b'o', b'g'])),
		ETestPair::from(("Marek", vec![0x85, b'M', b'a', b'r', b'e', b'k'])),
		ETestPair::from(("", hex!("80"))),
		ETestPair::from((
			"Lorem ipsum dolor sit amet, consectetur adipisicing elit",
			vec![
				0xb8, 0x38, b'L', b'o', b'r', b'e', b'm', b' ', b'i', b'p', b's', b'u', b'm', b' ', b'd', b'o', b'l',
				b'o', b'r', b' ', b's', b'i', b't', b' ', b'a', b'm', b'e', b't', b',', b' ', b'c', b'o', b'n', b's',
				b'e', b'c', b't', b'e', b't', b'u', b'r', b' ', b'a', b'd', b'i', b'p', b'i', b's', b'i', b'c', b'i',
				b'n', b'g', b' ', b'e', b'l', b'i', b't',
			],
		)),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_into_existing_buffer() {
	let mut buffer = BytesMut::new();
	buffer.extend_from_slice(b"junk");

	let mut split_buffer = buffer.split_off(buffer.len());
	split_buffer.extend_from_slice(b"!");

	let mut s = RlpStream::new_with_buffer(split_buffer);
	s.append(&"cat");
	buffer.unsplit(s.out());

	buffer.extend_from_slice(b" and ");

	let mut s = RlpStream::new_with_buffer(buffer);
	s.append(&"dog");
	let buffer = s.out();

	assert_eq!(
		&buffer[..],
		&[b'j', b'u', b'n', b'k', b'!', 0x83, b'c', b'a', b't', b' ', b'a', b'n', b'd', b' ', 0x83, b'd', b'o', b'g']
	);
}

#[test]
fn encode_address() {
	let tests = vec![ETestPair::from((
		H160::from(hex!("ef2d6d194084c2de36e0dabfce45d046b37d1106")),
		hex!("94ef2d6d194084c2de36e0dabfce45d046b37d1106"),
	))];
	run_encode_tests(tests);
}

/// Vec<u8> (Bytes) is treated as a single value
#[test]
fn encode_vector_u8() {
	let tests = vec![
		ETestPair::from((vec![], hex!("80"))),
		ETestPair::from((vec![0u8], hex!("00"))),
		ETestPair::from((vec![0x15], hex!("15"))),
		ETestPair::from((vec![0x40, 0x00], hex!("824000"))),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_bytes() {
	let tests = vec![
		ETestPair::from((Bytes::from_static(&hex!("")), hex!("80"))),
		ETestPair::from((Bytes::from_static(&hex!("00")), hex!("00"))),
		ETestPair::from((Bytes::from_static(&hex!("15")), hex!("15"))),
		ETestPair::from((Bytes::from_static(&hex!("4000")), hex!("824000"))),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_bytesmut() {
	let tests = vec![
		ETestPair::from((BytesMut::from(&[] as &[u8]), hex!("80"))),
		ETestPair::from((BytesMut::from(&hex!("00") as &[u8]), hex!("00"))),
		ETestPair::from((BytesMut::from(&hex!("15") as &[u8]), hex!("15"))),
		ETestPair::from((BytesMut::from(&hex!("4000") as &[u8]), hex!("824000"))),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_vector_u64() {
	let tests = vec![
		VETestPair::from((vec![], hex!("c0"))),
		VETestPair::from((vec![15_u64], hex!("c10f"))),
		VETestPair::from((vec![1, 2, 3, 7, 0xff], hex!("c60102030781ff"))),
		VETestPair::from((vec![0xffff_ffff, 1, 2, 3, 7, 0xff], hex!("cb84ffffffff0102030781ff"))),
	];
	run_encode_tests_list(tests);
}

#[test]
fn encode_vector_str() {
	let tests = vec![VETestPair(vec!["cat", "dog"], vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'])];
	run_encode_tests_list(tests);
}

#[test]
fn clear() {
	let mut buffer = BytesMut::new();
	buffer.extend_from_slice(b"junk");

	let mut s = RlpStream::new_with_buffer(buffer);
	s.append(&"parrot");
	s.clear();
	s.append(&"cat");

	assert_eq!(&s.out()[..], &[b'j', b'u', b'n', b'k', 0x83, b'c', b'a', b't']);
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

impl<T, Repr> From<(T, Repr)> for DTestPair<T>
where
	T: Decodable + fmt::Debug + cmp::Eq,
	Repr: Into<Vec<u8>>,
{
	fn from((v, repr): (T, Repr)) -> Self {
		Self(v, repr.into())
	}
}

impl<T, Repr> From<(Vec<T>, Repr)> for VDTestPair<T>
where
	T: Decodable + fmt::Debug + cmp::Eq,
	Repr: Into<Vec<u8>>,
{
	fn from((v, repr): (Vec<T>, Repr)) -> Self {
		Self(v, repr.into())
	}
}

/// Vec<u8> (Bytes) is treated as a single value
#[test]
fn decode_vector_u8() {
	let tests = vec![
		DTestPair::from((vec![], hex!("80"))),
		DTestPair::from((vec![0_u8], hex!("00"))),
		DTestPair::from((vec![0x15], hex!("15"))),
		DTestPair::from((vec![0x40, 0x00], hex!("824000"))),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_bytes() {
	let tests = vec![
		DTestPair::from((Bytes::from_static(&hex!("")), hex!("80"))),
		DTestPair::from((Bytes::from_static(&hex!("00")), hex!("00"))),
		DTestPair::from((Bytes::from_static(&hex!("15")), hex!("15"))),
		DTestPair::from((Bytes::from_static(&hex!("4000")), hex!("824000"))),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_bytesmut() {
	let tests = vec![
		DTestPair::from((BytesMut::from(&hex!("") as &[u8]), hex!("80"))),
		DTestPair::from((BytesMut::from(&hex!("00") as &[u8]), hex!("00"))),
		DTestPair::from((BytesMut::from(&hex!("15") as &[u8]), hex!("15"))),
		DTestPair::from((BytesMut::from(&hex!("4000") as &[u8]), hex!("824000"))),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_u8() {
	let tests = vec![
		DTestPair::from((0x0_u8, hex!("80"))),
		DTestPair::from((0x77_u8, hex!("77"))),
		DTestPair::from((0xcc_u8, hex!("81cc"))),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_u16() {
	let tests = vec![DTestPair::from((0x100u16, hex!("820100"))), DTestPair::from((0xffffu16, hex!("82ffff")))];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_u32() {
	let tests =
		vec![DTestPair::from((0x0001_0000u32, hex!("83010000"))), DTestPair::from((0x00ff_ffffu32, hex!("83ffffff")))];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_u64() {
	let tests = vec![
		DTestPair::from((0x0100_0000_u64, hex!("8401000000"))),
		DTestPair::from((0xFFFF_FFFF_u64, hex!("84ffffffff"))),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_u128() {
	let tests = vec![
		DTestPair::from((0x0100_0000_0000_0000_u128, hex!("880100000000000000"))),
		DTestPair::from((0xFFFF_FFFF_FFFF_FFFF_u128, hex!("88ffffffffffffffff"))),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_u256() {
	let tests = vec![
		DTestPair::from((U256::from(0_u64), hex!("80"))),
		DTestPair::from((U256::from(0x0100_0000_u64), hex!("8401000000"))),
		DTestPair::from((U256::from(0xffff_ffff_u64), hex!("84ffffffff"))),
		DTestPair::from((
			hex!("  8090a0b0c0d0e0f00910203040506077000000000000000100000000000012f0").into(),
			hex!("a08090a0b0c0d0e0f00910203040506077000000000000000100000000000012f0"),
		)),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_str() {
	let tests = vec![
		DTestPair::from(("cat".to_owned(), vec![0x83, b'c', b'a', b't'])),
		DTestPair::from(("dog".to_owned(), vec![0x83, b'd', b'o', b'g'])),
		DTestPair::from(("Marek".to_owned(), vec![0x85, b'M', b'a', b'r', b'e', b'k'])),
		DTestPair::from(("".to_owned(), hex!("80"))),
		DTestPair::from((
			"Lorem ipsum dolor sit amet, consectetur adipisicing elit".to_owned(),
			vec![
				0xb8, 0x38, b'L', b'o', b'r', b'e', b'm', b' ', b'i', b'p', b's', b'u', b'm', b' ', b'd', b'o', b'l',
				b'o', b'r', b' ', b's', b'i', b't', b' ', b'a', b'm', b'e', b't', b',', b' ', b'c', b'o', b'n', b's',
				b'e', b'c', b't', b'e', b't', b'u', b'r', b' ', b'a', b'd', b'i', b'p', b'i', b's', b'i', b'c', b'i',
				b'n', b'g', b' ', b'e', b'l', b'i', b't',
			],
		)),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_address() {
	let tests = vec![DTestPair::from((
		H160::from(hex!("ef2d6d194084c2de36e0dabfce45d046b37d1106")),
		hex!("94ef2d6d194084c2de36e0dabfce45d046b37d1106"),
	))];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_vector_u64() {
	let tests = vec![
		VDTestPair::from((vec![], hex!("c0"))),
		VDTestPair::from((vec![15_u64], hex!("c10f"))),
		VDTestPair::from((vec![1, 2, 3, 7, 0xff], hex!("c60102030781ff"))),
		VDTestPair::from((vec![0xffff_ffff, 1, 2, 3, 7, 0xff], hex!("cb84ffffffff0102030781ff"))),
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
	let mut data = hex!("b8ff").to_vec();
	for _ in 0..253 {
		data.push(b'c');
	}

	let rlp = Rlp::new(&data);

	let as_val: Result<String, DecoderError> = rlp.as_val();
	assert_eq!(Err(DecoderError::RlpInconsistentLengthAndData), as_val);
}

#[test]
fn test_the_exact_long_string() {
	let mut data = hex!("b8ff").to_vec();
	for _ in 0..255 {
		data.push(b'c');
	}

	let rlp = Rlp::new(&data);

	let as_val: Result<String, DecoderError> = rlp.as_val();
	assert!(as_val.is_ok());
}

#[test]
fn test_rlp_2bytes_data_length_check() {
	let mut data = hex!("b902ff").to_vec(); // 512+255
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
	stream.append(&0x28_u32);
	assert_eq!(stream.out()[..], hex!("c2c028")[..]);
}

#[test]
fn test_rlp_list_length_overflow() {
	let data = hex!("ffffffffffffffffff000000");
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
		assert_eq!(stream.out().len(), limit);
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
		assert!(!rlp.is_int());
	}
}

#[test]
fn test_bool_same_as_int() {
	assert_eq!(rlp::encode(&false), rlp::encode(&0x00u8));
	assert_eq!(rlp::encode(&true), rlp::encode(&0x01u8));
	let two = rlp::encode(&0x02u8);
	let invalid: Result<bool, _> = rlp::decode(&two);
	invalid.unwrap_err();
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
			s.begin_unbounded_list()
				.append(&self.0)
				.append(&self.1)
				.finalize_unbounded_list();
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
