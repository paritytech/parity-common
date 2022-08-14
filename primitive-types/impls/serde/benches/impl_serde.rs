// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! benchmarking for impl_serde
//! should be started with:
//! ```bash
//! cargo bench
//! ```

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use impl_serde::impl_uint_serde;
use serde_derive::{Deserialize, Serialize};
use uint::*;

mod input;

construct_uint! {
	pub struct U256(4);
}

impl_uint_serde!(U256, 4);

#[derive(Debug, Deserialize, Serialize)]
struct Bytes(#[serde(with = "impl_serde::serialize")] Vec<u8>);

criterion_group!(impl_serde, u256_to_hex, hex_to_u256, bytes_to_hex, hex_to_bytes,);
criterion_main!(impl_serde);

fn u256_to_hex(c: &mut Criterion) {
	let mut group = c.benchmark_group("u256_to_hex");
	for input in [
		U256::from(0),
		U256::from(100),
		U256::from(u32::max_value()),
		U256::from(u64::max_value()),
		U256::from(u128::max_value()),
		U256([1, 2, 3, 4]),
	] {
		group.bench_with_input(BenchmarkId::from_parameter(input), &input, |b, x| {
			b.iter(|| black_box(serde_json::to_string(&x)))
		});
	}
	group.finish();
}

fn hex_to_u256(c: &mut Criterion) {
	let mut group = c.benchmark_group("hex_to_u256");
	for input in [
		"\"0x0\"",
		"\"0x1\"",
		"\"0x10\"",
		"\"0x100\"",
		"\"0x1000000000000000000000000000000000000000000000000000000000000100\"",
	] {
		group.bench_with_input(BenchmarkId::from_parameter(input), &input, |b, x| {
			b.iter(|| black_box(serde_json::from_str::<U256>(&x)))
		});
	}
	group.finish();
}

fn bytes_to_hex(c: &mut Criterion) {
	let mut group = c.benchmark_group("bytes_to_hex");
	let params = [
		input::HEX_64_CHARS,
		input::HEX_256_CHARS,
		input::HEX_1024_CHARS,
		input::HEX_4096_CHARS,
		input::HEX_16384_CHARS,
		input::HEX_65536_CHARS,
	];
	for param in params {
		let input = serde_json::from_str::<Bytes>(&param).unwrap();
		group.bench_with_input(BenchmarkId::from_parameter(param.len()), &input, |b, x| {
			b.iter(|| black_box(serde_json::to_string(&x)))
		});
	}
	group.finish();
}

fn hex_to_bytes(c: &mut Criterion) {
	let mut group = c.benchmark_group("hex_to_bytes");
	for input in [
		input::HEX_64_CHARS,
		input::HEX_256_CHARS,
		input::HEX_1024_CHARS,
		input::HEX_4096_CHARS,
		input::HEX_16384_CHARS,
		input::HEX_65536_CHARS,
	] {
		group.bench_with_input(BenchmarkId::from_parameter(input.len()), &input, |b, x| {
			b.iter(|| black_box(serde_json::from_str::<Bytes>(&x)))
		});
	}
	group.finish();
}
