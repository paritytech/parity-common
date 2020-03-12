// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! benchmarking for rlp

use criterion::{criterion_group, criterion_main, Criterion};

fn bench_encode(c: &mut Criterion) {
	c.bench_function("encode_u64", |b| {
		b.iter(|| {
			let mut stream = rlp::RlpStream::new();
			stream.append(&0x1023_4567_89ab_cdefu64);
			let _ = stream.out();
		})
	});
	c.bench_function("encode_u256", |b| {
		b.iter(|| {
			let mut stream = rlp::RlpStream::new();
			let uint: primitive_types::U256 = "8090a0b0c0d0e0f00910203040506077000000000000000100000000000012f0".into();
			stream.append(&uint);
			let _ = stream.out();
		})
	});
	c.bench_function("encode_1000_u64", |b| {
		b.iter(|| {
			let mut stream = rlp::RlpStream::new_list(1000);
			for i in 0..1000u64 {
				stream.append(&i);
			}
			let _ = stream.out();
		})
	});
	c.bench_function("encode_nested_empty_lists", |b| {
		b.iter(|| {
			// [ [], [[]], [ [], [[]] ] ]
			let mut stream = rlp::RlpStream::new_list(3);
			stream.begin_list(0);
			stream.begin_list(1).begin_list(0);
			stream.begin_list(2).begin_list(0).begin_list(1).begin_list(0);
			let _ = stream.out();
		})
	});
	c.bench_function("encode_1000_empty_lists", |b| {
		b.iter(|| {
			let mut stream = rlp::RlpStream::new_list(1000);
			for _ in 0..1000 {
				stream.begin_list(0);
			}
			let _ = stream.out();
		})
	});
}

fn bench_decode(c: &mut Criterion) {
	c.bench_function("decode_u64", |b| {
		b.iter(|| {
			let data = vec![0x88, 0x10, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
			let rlp = rlp::Rlp::new(&data);
			let _: u64 = rlp.as_val().unwrap();
		})
	});
	c.bench_function("decode_u256", |b| {
		b.iter(|| {
			let data = vec![
				0xa0, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0, 0x09, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x77,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0xf0,
			];
			let rlp = rlp::Rlp::new(&data);
			let _: primitive_types::U256 = rlp.as_val().unwrap();
		})
	});
	c.bench_function("decode_1000_u64", |b| {
		let mut stream = rlp::RlpStream::new_list(1000);
		for i in 0..1000u64 {
			stream.append(&i);
		}
		let data = stream.out();
		b.iter(|| {
			let rlp = rlp::Rlp::new(&data);
			for i in 0..1000 {
				let _: u64 = rlp.val_at(i).unwrap();
			}
		});
	});
	c.bench_function("decode_nested_empty_lists", |b| {
		b.iter(|| {
			// [ [], [[]], [ [], [[]] ] ]
			let data = vec![0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0];
			let rlp = rlp::Rlp::new(&data);
			let _v0: Vec<u16> = rlp.at(0).unwrap().as_list().unwrap();
			let _v1: Vec<u16> = rlp.at(1).unwrap().at(0).unwrap().as_list().unwrap();
			let nested_rlp = rlp.at(2).unwrap();
			let _v2a: Vec<u16> = nested_rlp.at(0).unwrap().as_list().unwrap();
			let _v2b: Vec<u16> = nested_rlp.at(1).unwrap().at(0).unwrap().as_list().unwrap();
		})
	});
	c.bench_function("decode_1000_empty_lists", |b| {
		let mut stream = rlp::RlpStream::new_list(1000);
		for _ in 0..1000 {
			stream.begin_list(0);
		}
		let data = stream.out();
		b.iter(|| {
			let rlp = rlp::Rlp::new(&data);
			for i in 0..1000 {
				let _: Vec<u8> = rlp.at(i).unwrap().as_list().unwrap();
			}
		});
	});
}

criterion_group!(benches, bench_encode, bench_decode);
criterion_main!(benches);
