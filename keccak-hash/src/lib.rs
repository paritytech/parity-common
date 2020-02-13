// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use std::io;

pub use primitive_types::H256;
use tiny_keccak::{Hasher, Keccak};

/// Get the KECCAK (i.e. Keccak) hash of the empty bytes string.
pub const KECCAK_EMPTY: H256 = H256([
	0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0, 0xe5, 0x00, 0xb6,
	0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70,
]);

/// The KECCAK of the RLP encoding of empty data.
pub const KECCAK_NULL_RLP: H256 = H256([
	0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e, 0x5b, 0x48, 0xe0,
	0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21,
]);

/// The KECCAK of the RLP encoding of empty list.
pub const KECCAK_EMPTY_LIST_RLP: H256 = H256([
	0x1d, 0xcc, 0x4d, 0xe8, 0xde, 0xc7, 0x5d, 0x7a, 0xab, 0x85, 0xb5, 0x67, 0xb6, 0xcc, 0xd4, 0x1a, 0xd3, 0x12, 0x45,
	0x1b, 0x94, 0x8a, 0x74, 0x13, 0xf0, 0xa1, 0x42, 0xfd, 0x40, 0xd4, 0x93, 0x47,
]);

pub fn keccak<T: AsRef<[u8]>>(s: T) -> H256 {
	let mut result = [0u8; 32];
	write_keccak(s, &mut result);
	H256(result)
}

/// Computes in-place keccak256 hash of `data`.
pub fn keccak256(data: &mut [u8]) {
	let mut keccak256 = Keccak::v256();
	keccak256.update(data.as_ref());
	keccak256.finalize(data);
}

/// Computes in-place keccak512 hash of `data`.
pub fn keccak512(data: &mut [u8]) {
	let mut keccak512 = Keccak::v512();
	keccak512.update(data.as_ref());
	keccak512.finalize(data);
}

pub fn keccak_256(input: &[u8], output: &mut [u8]) {
	write_keccak(input, output);
}

pub fn keccak_512(input: &[u8], output: &mut [u8]) {
	let mut keccak512 = Keccak::v512();
	keccak512.update(input);
	keccak512.finalize(output);
}

pub fn write_keccak<T: AsRef<[u8]>>(s: T, dest: &mut [u8]) {
	let mut keccak256 = Keccak::v256();
	keccak256.update(s.as_ref());
	keccak256.finalize(dest);
}

#[cfg(feature = "std")]
pub fn keccak_pipe(r: &mut dyn io::BufRead, w: &mut dyn io::Write) -> Result<H256, io::Error> {
	let mut output = [0u8; 32];
	let mut input = [0u8; 1024];
	let mut keccak256 = Keccak::v256();

	// read file
	loop {
		let some = r.read(&mut input)?;
		if some == 0 {
			break;
		}
		keccak256.update(&input[0..some]);
		w.write_all(&input[0..some])?;
	}

	keccak256.finalize(&mut output);
	Ok(output.into())
}

#[cfg(feature = "std")]
pub fn keccak_buffer(r: &mut dyn io::BufRead) -> Result<H256, io::Error> {
	keccak_pipe(r, &mut io::sink())
}

#[cfg(test)]
mod tests {
	#[cfg(not(feature = "std"))]
	extern crate alloc;
	#[cfg(not(feature = "std"))]
	use alloc::{vec, vec::Vec};

	use super::*;

	#[test]
	fn keccak_empty() {
		assert_eq!(keccak([0u8; 0]), KECCAK_EMPTY);
	}

	#[test]
	fn keccak_as() {
		assert_eq!(
			keccak([0x41u8; 32]),
			H256([
				0x59, 0xca, 0xd5, 0x94, 0x86, 0x73, 0x62, 0x2c, 0x1d, 0x64, 0xe2, 0x32, 0x24, 0x88, 0xbf, 0x01, 0x61,
				0x9f, 0x7f, 0xf4, 0x57, 0x89, 0x74, 0x1b, 0x15, 0xa9, 0xf7, 0x82, 0xce, 0x92, 0x90, 0xa8
			]),
		);
	}

	#[test]
	fn write_keccak_with_content() {
		let data: Vec<u8> = From::from("hello world");
		let expected = vec![
			0x47, 0x17, 0x32, 0x85, 0xa8, 0xd7, 0x34, 0x1e, 0x5e, 0x97, 0x2f, 0xc6, 0x77, 0x28, 0x63, 0x84, 0xf8, 0x02,
			0xf8, 0xef, 0x42, 0xa5, 0xec, 0x5f, 0x03, 0xbb, 0xfa, 0x25, 0x4c, 0xb0, 0x1f, 0xad,
		];
		let mut dest = [0u8; 32];
		write_keccak(data, &mut dest);

		assert_eq!(dest, expected.as_ref());
	}

	#[cfg(feature = "std")]
	#[test]
	fn should_keccak_a_file() {
		use std::fs;
		use std::io::{BufReader, Write};

		// given
		let tmpdir = tempdir::TempDir::new("keccak").unwrap();
		let mut path = tmpdir.path().to_owned();
		path.push("should_keccak_a_file");
		// Prepare file
		{
			let mut file = fs::File::create(&path).unwrap();
			file.write_all(b"something").unwrap();
		}

		let mut file = BufReader::new(fs::File::open(&path).unwrap());
		// when
		let hash = keccak_buffer(&mut file).unwrap();

		// then
		assert_eq!(format!("{:x}", hash), "68371d7e884c168ae2022c82bd837d51837718a7f7dfb7aa3f753074a35e1d87");
	}
}
