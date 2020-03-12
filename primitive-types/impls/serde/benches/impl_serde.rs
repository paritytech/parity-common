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

use criterion::{black_box, criterion_group, criterion_main, Criterion, ParameterizedBenchmark};
use serde_derive::{Deserialize, Serialize};
// TODO(niklasad1): use `uint::construct_uint` when a new version of `uint` is released
use impl_serde::impl_uint_serde;
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
	c.bench(
		"u256_to_hex",
		ParameterizedBenchmark::new(
			"",
			|b, x| b.iter(|| black_box(serde_json::to_string(&x))),
			vec![
				U256::from(0),
				U256::from(100),
				U256::from(u32::max_value()),
				U256::from(u64::max_value()),
				U256::from(u128::max_value()),
				U256([1, 2, 3, 4]),
			],
		),
	);
}

fn hex_to_u256(c: &mut Criterion) {
	let parameters = vec![
		r#""0x0""#,
		r#""0x1""#,
		r#""0x10""#,
		r#""0x100""#,
		r#""0x1000000000000000000000000000000000000000000000000000000000000100""#,
	];

	c.bench(
		"hex_to_u256",
		ParameterizedBenchmark::new("", |b, x| b.iter(|| black_box(serde_json::from_str::<U256>(&x))), parameters),
	);
}

fn bytes_to_hex(c: &mut Criterion) {
	let parameters = vec![
		serde_json::from_str::<Bytes>(&input::HEX_64_CHARS).unwrap(),
		serde_json::from_str::<Bytes>(&input::HEX_256_CHARS).unwrap(),
		serde_json::from_str::<Bytes>(&input::HEX_1024_CHARS).unwrap(),
		serde_json::from_str::<Bytes>(&input::HEX_4096_CHARS).unwrap(),
		serde_json::from_str::<Bytes>(&input::HEX_16384_CHARS).unwrap(),
		serde_json::from_str::<Bytes>(&input::HEX_65536_CHARS).unwrap(),
	];

	c.bench(
		"bytes to hex",
		ParameterizedBenchmark::new("", |b, x| b.iter(|| black_box(serde_json::to_string(&x))), parameters),
	);
}

fn hex_to_bytes(c: &mut Criterion) {
	let parameters = vec![
		input::HEX_64_CHARS,
		input::HEX_256_CHARS,
		input::HEX_1024_CHARS,
		input::HEX_4096_CHARS,
		input::HEX_16384_CHARS,
		input::HEX_65536_CHARS,
	];

	c.bench(
		"hex to bytes",
		ParameterizedBenchmark::new("", |b, x| b.iter(|| black_box(serde_json::from_str::<Bytes>(&x))), parameters),
	);
}
