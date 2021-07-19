// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! benchmarking for bigint
//! should be started with:
//! ```bash
//! rustup run cargo bench
//! ```

use criterion::{criterion_group, criterion_main};
use uint::{construct_uint, uint_full_mul_reg};

construct_uint! {
	pub struct U256(4);
}

construct_uint! {
	pub struct U512(8);
}

impl U256 {
	#[inline(always)]
	pub fn full_mul(self, other: U256) -> U512 {
		U512(uint_full_mul_reg!(U256, 4, self, other))
	}
}

use criterion::{black_box, Bencher, Criterion, ParameterizedBenchmark};
use num_bigint::BigUint;
use rug::{integer::Order, Integer};
use std::str::FromStr;

criterion_group!(
	bigint,
	u256_add,
	u256_sub,
	u256_mul,
	u256_mul_full,
	u256_div,
	u512_div_mod,
	u256_rem,
	u256_integer_sqrt,
	u256_bit_and,
	u256_bit_or,
	u256_bit_xor,
	u256_not,
	u256_ord,
	u256_shl,
	u256_shr,
	u256_from_le,
	u256_from_be,
	u512_add,
	u512_sub,
	u512_mul,
	u512_div,
	u512_rem,
	u512_integer_sqrt,
	u512_mul_u32_vs_u64,
	mulmod_u512_vs_biguint_vs_gmp,
	conversions,
	u512_bit_and,
	u512_bit_or,
	u512_bit_xor,
	u512_not,
	u512_ord,
	u512_shl,
	u512_shr,
	u128_mul,
	u128_div,
	from_fixed_array,
	from_str,
);
criterion_main!(bigint);

fn to_biguint(x: U256) -> BigUint {
	let mut bytes = [0u8; 32];
	x.to_little_endian(&mut bytes);
	BigUint::from_bytes_le(&bytes)
}

fn from_biguint(x: BigUint) -> U512 {
	let bytes = x.to_bytes_le();
	U512::from_little_endian(&bytes)
}

fn to_gmp(x: U256) -> Integer {
	let U256(ref arr) = x;
	Integer::from_digits(&arr[..], Order::Lsf)
}

fn from_gmp(x: Integer) -> U512 {
	let digits = x.to_digits(Order::LsfLe);
	U512::from_little_endian(&digits)
}

fn u128_div(c: &mut Criterion) {
	c.bench(
		"u128_div",
		ParameterizedBenchmark::new(
			"",
			|b, (x, y, z)| {
				b.iter(|| {
					let x = black_box(u128::from(*x) << 64 + u128::from(*y));
					black_box(x / u128::from(*z))
				})
			},
			vec![(0u64, u64::max_value(), 100u64), (u64::max_value(), u64::max_value(), 99), (42, 42, 100500)],
		),
	);
}

fn u256_add(c: &mut Criterion) {
	c.bench(
		"u256_add",
		ParameterizedBenchmark::new(
			"",
			|b, (x, y)| {
				b.iter(|| {
					let x = U256::from(*x);
					let y = U256::from(*y);
					black_box(x.overflowing_add(y).0)
				})
			},
			vec![(0u64, 1u64), (u64::max_value(), 1), (42, 100500)],
		),
	);
}

fn u256_sub(c: &mut Criterion) {
	c.bench(
		"u256_sub",
		ParameterizedBenchmark::new(
			"",
			|b, (x, y)| {
				b.iter(|| {
					let y = U256::from(*y);
					black_box(x.overflowing_sub(y).0)
				})
			},
			vec![(U256::max_value(), 1u64), (U256::from(3), 2)],
		),
	);
}

fn u256_mul(c: &mut Criterion) {
	c.bench(
		"u256_mul",
		ParameterizedBenchmark::new(
			"",
			|b, (x, y)| {
				b.iter(|| {
					let y = U256::from(*y);
					black_box(x.overflowing_mul(y).0)
				})
			},
			vec![
				(U256::max_value(), 1u64),
				(U256::from(3), u64::max_value()),
				(U256::from_dec_str("21674844646682989462120101885968193938394323990565507610662749").unwrap(), 173),
			],
		),
	);
}

fn u512_div_mod(c: &mut Criterion) {
	c.bench(
		"u512_div_mod",
		ParameterizedBenchmark::new(
			"",
			|b, (x, y)| {
				b.iter(|| {
					let (q, r) = x.div_mod(*y);
					black_box((q, r))
				})
			},
			vec![
				(U512::max_value(), U512::from(1u64)),
				(U512::from(u64::max_value()), U512::from(u32::max_value())),
				(U512::from(u64::max_value()), U512::from(u64::max_value() - 1)),
				(U512::from(u64::max_value()), U512::from(u64::max_value() - 1)),
				(
					U512::from_dec_str("3759751734479964094783137206182536765532905409829204647089173492").unwrap(),
					U512::from_dec_str("21674844646682989462120101885968193938394323990565507610662749").unwrap(),
				),
				(
					U512::from_str(
						"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
					)
						.unwrap(),
					U512::from_str(
						"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0",
					)
						.unwrap(),
				),
				(
					U512::from_dec_str(
						"204586912993508866875824356051724947013540127877691549342705710506008362274387533983037847993622361501550043477868832682875761627559574690771211649025"
					).unwrap(),
					U512::from_dec_str(
						"452312848583266388373324160190187140051835877600158453279131187530910662640"
					).unwrap(),
				),
			],
		),
	);
}

fn u256_mul_full(c: &mut Criterion) {
	c.bench(
		"u256_mul_full",
		ParameterizedBenchmark::new(
			"",
			|b, (x, y)| {
				b.iter(|| {
					let y = *y;
					let U512(ref u512words) = x.full_mul(U256([y, y, y, y]));
					black_box(U256([u512words[0], u512words[2], u512words[2], u512words[3]]))
				})
			},
			vec![(U256::from(42), 1u64), (U256::from(3), u64::max_value())],
		),
	);
}

fn u256_div(c: &mut Criterion) {
	let one = U256([12767554894655550452, 16333049135534778834, 140317443000293558, 598963]);
	let two = U256([2096410819092764509, 8483673822214032535, 36306297304129857, 3453]);
	c.bench_function("u256_div", move |b| b.iter(|| black_box(one / two)));
}

fn u256_rem(c: &mut Criterion) {
	c.bench(
		"u256_rem",
		ParameterizedBenchmark::new(
			"",
			|b, (x, y)| b.iter(|| black_box(x % y)),
			vec![
				(U256::max_value(), U256::from(1u64)),
				(U256::from(u64::max_value()), U256::from(u64::from(u32::max_value()) + 1)),
				(
					U256([12767554894655550452, 16333049135534778834, 140317443000293558, 598963]),
					U256([2096410819092764509, 8483673822214032535, 36306297304129857, 3453]),
				),
				(
					U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(),
					U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0").unwrap(),
				),
			],
		),
	);
}

fn u256_integer_sqrt(c: &mut Criterion) {
	c.bench(
		"u256_integer_sqrt",
		ParameterizedBenchmark::new(
			"",
			|b, x| b.iter(|| black_box(x.integer_sqrt().0)),
			vec![
				U256::from(u64::MAX),
				U256::from(u128::MAX) + 1,
				U256::from(u128::MAX - 1) * U256::from(u128::MAX - 1) - 1,
				U256::MAX,
			],
		),
	);
}

fn u512_pairs() -> Vec<(U512, U512)> {
	vec![
		(U512::from(1u64), U512::from(0u64)),
		(U512::from(u64::max_value()), U512::from(u64::from(u32::max_value()) + 1)),
		(
			U512([12767554894655550452, 16333049135534778834, 140317443000293558, 598963, 0, 0, 0, 0]),
			U512([0, 0, 0, 0, 2096410819092764509, 8483673822214032535, 36306297304129857, 3453]),
		),
		(
			U512::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap(),
			U512::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0").unwrap(),
		),
	]
}

fn u512_add(c: &mut Criterion) {
	c.bench("u512_add", ParameterizedBenchmark::new("", |b, (x, y)| b.iter(|| black_box(x + y)), u512_pairs()));
}

fn u512_sub(c: &mut Criterion) {
	c.bench(
		"u512_sub",
		ParameterizedBenchmark::new("", |b, (x, y)| b.iter(|| black_box(x.overflowing_sub(*y).0)), u512_pairs()),
	);
}

fn u512_mul(c: &mut Criterion) {
	c.bench(
		"u512_mul",
		ParameterizedBenchmark::new("", |b, (x, y)| b.iter(|| black_box(x.overflowing_mul(*y).0)), u512_pairs()),
	);
}

fn u512_integer_sqrt(c: &mut Criterion) {
	c.bench(
		"u512_integer_sqrt",
		ParameterizedBenchmark::new(
			"",
			|b, x| b.iter(|| black_box(x.integer_sqrt().0)),
			vec![
				U512::from(u32::MAX) + 1,
				U512::from(u64::MAX),
				(U512::from(u128::MAX) + 1) * (U512::from(u128::MAX) + 1),
				U256::MAX.full_mul(U256::MAX) - 1,
				U512::MAX,
			],
		),
	);
}

fn u512_div(c: &mut Criterion) {
	let one = U512([
		8326634216714383706,
		15837136097609390493,
		13004317189126203332,
		7031796866963419685,
		12767554894655550452,
		16333049135534778834,
		140317443000293558,
		598963,
	]);
	let two = U512([
		11707750893627518758,
		17679501210898117940,
		2472932874039724966,
		11177683849610900539,
		2096410819092764509,
		8483673822214032535,
		36306297304129857,
		3453,
	]);
	c.bench_function("u512_div", move |b| b.iter(|| black_box(one / two)));
}

fn u512_rem(c: &mut Criterion) {
	let one = U512([
		8326634216714383706,
		15837136097609390493,
		13004317189126203332,
		7031796866963419685,
		12767554894655550452,
		16333049135534778834,
		140317443000293558,
		598963,
	]);
	let two = U512([
		11707750893627518758,
		17679501210898117940,
		2472932874039724966,
		11177683849610900539,
		2096410819092764509,
		8483673822214032535,
		36306297304129857,
		3453,
	]);
	c.bench_function("u512_rem", move |b| b.iter(|| black_box(one % two)));
}

fn conversions(c: &mut Criterion) {
	c.bench(
		"conversions biguint vs gmp",
		ParameterizedBenchmark::new("BigUint", |b, i| bench_convert_to_biguit(b, *i), vec![0, 42, u64::max_value()])
			.with_function("gmp", |b, i| bench_convert_to_gmp(b, *i)),
	);
}

fn bench_convert_to_biguit(b: &mut Bencher, i: u64) {
	let z = U256::from(i);
	let z512 = U512::from(i);
	b.iter(|| {
		let zb = to_biguint(z);
		assert_eq!(from_biguint(zb), z512);
	});
}

fn bench_convert_to_gmp(b: &mut Bencher, i: u64) {
	let z = U256::from(i);
	let z512 = U512::from(i);
	b.iter(|| {
		let zb = to_gmp(z);
		assert_eq!(from_gmp(zb), z512);
	});
}

fn u512_mul_u32_vs_u64(c: &mut Criterion) {
	let ms = vec![1u32, 42, 10_000_001, u32::max_value()];
	c.bench(
		"multiply u512 by u32 vs u64",
		ParameterizedBenchmark::new("u32", |b, i| bench_u512_mul_u32(b, *i), ms)
			.with_function("u64", |b, i| bench_u512_mul_u64(b, u64::from(*i))),
	);
}

fn bench_u512_mul_u32(b: &mut Bencher, i: u32) {
	let x = U512::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
	b.iter(|| black_box(x * i));
}

fn bench_u512_mul_u64(b: &mut Bencher, i: u64) {
	let x = U512::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
	b.iter(|| black_box(x * i));
}

fn mulmod_u512_vs_biguint_vs_gmp(c: &mut Criterion) {
	let mods = vec![
		U256::from(1u64),
		U256::from(10_000_001u64),
		U256::from(u64::max_value()),
		U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF1").unwrap(),
	];
	c.bench(
		"mulmod u512 vs biguint vs gmp",
		ParameterizedBenchmark::new("u512", |b, i| bench_u512_mulmod(b, *i), mods)
			.with_function("BigUint", |b, i| bench_biguint_mulmod(b, *i))
			.with_function("gmp", |b, i| bench_gmp_mulmod(b, *i)),
	);
}

fn bench_biguint_mulmod(b: &mut Bencher, z: U256) {
	let x = U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
	let y = U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
	b.iter(|| {
		let w = to_biguint(x) * to_biguint(y);
		black_box(from_biguint(w % to_biguint(z)))
	});
}

fn bench_gmp_mulmod(b: &mut Bencher, z: U256) {
	let x = U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
	let y = U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
	b.iter(|| {
		let w = to_gmp(x) * to_gmp(y);
		black_box(from_gmp(w % to_gmp(z)))
	});
}

fn bench_u512_mulmod(b: &mut Bencher, z: U256) {
	let x = U512::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
	let y = U512::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
	let z = U512([z.0[0], z.0[1], z.0[2], z.0[3], 0, 0, 0, 0]);
	b.iter(|| {
		let w = x.overflowing_mul(y).0;
		black_box(w % z)
	});
}

// NOTE: uses native `u128` and does not measure this crates performance,
// but might be interesting as a comparison.
fn u128_mul(c: &mut Criterion) {
	c.bench_function("u128_mul", |b| b.iter(|| black_box(12345u128 * u128::from(u64::max_value()))));
}

fn u256_bit_and(c: &mut Criterion) {
	let one = U256([12767554894655550452, 16333049135534778834, 140317443000293558, 598963]);
	let two = U256([2096410819092764509, 8483673822214032535, 36306297304129857, 3453]);
	c.bench_function("u256_bit_and", move |b| b.iter(|| black_box(one & two)));
}

fn u512_bit_and(c: &mut Criterion) {
	let one = U512([
		8326634216714383706,
		15837136097609390493,
		13004317189126203332,
		7031796866963419685,
		12767554894655550452,
		16333049135534778834,
		140317443000293558,
		598963,
	]);
	let two = U512([
		11707750893627518758,
		17679501210898117940,
		2472932874039724966,
		11177683849610900539,
		2096410819092764509,
		8483673822214032535,
		36306297304129857,
		3453,
	]);
	c.bench_function("u512_bit_and", move |b| b.iter(|| black_box(one & two)));
}

fn u256_bit_xor(c: &mut Criterion) {
	let one = U256([12767554894655550452, 16333049135534778834, 140317443000293558, 598963]);
	let two = U256([2096410819092764509, 8483673822214032535, 36306297304129857, 3453]);
	c.bench_function("u256_bit_xor", move |b| b.iter(|| black_box(one ^ two)));
}

fn u512_bit_xor(c: &mut Criterion) {
	let one = U512([
		8326634216714383706,
		15837136097609390493,
		13004317189126203332,
		7031796866963419685,
		12767554894655550452,
		16333049135534778834,
		140317443000293558,
		598963,
	]);
	let two = U512([
		11707750893627518758,
		17679501210898117940,
		2472932874039724966,
		11177683849610900539,
		2096410819092764509,
		8483673822214032535,
		36306297304129857,
		3453,
	]);
	c.bench_function("u512_bit_xor", move |b| b.iter(|| black_box(one ^ two)));
}

fn u256_bit_or(c: &mut Criterion) {
	let one = U256([12767554894655550452, 16333049135534778834, 140317443000293558, 598963]);
	let two = U256([2096410819092764509, 8483673822214032535, 36306297304129857, 3453]);
	c.bench_function("u256_bit_or", move |b| b.iter(|| black_box(one | two)));
}

fn u512_bit_or(c: &mut Criterion) {
	let one = U512([
		8326634216714383706,
		15837136097609390493,
		13004317189126203332,
		7031796866963419685,
		12767554894655550452,
		16333049135534778834,
		140317443000293558,
		598963,
	]);
	let two = U512([
		11707750893627518758,
		17679501210898117940,
		2472932874039724966,
		11177683849610900539,
		2096410819092764509,
		8483673822214032535,
		36306297304129857,
		3453,
	]);
	c.bench_function("u512_bit_or", move |b| b.iter(|| black_box(one | two)));
}

fn u256_not(c: &mut Criterion) {
	let one = U256([12767554894655550452, 16333049135534778834, 140317443000293558, 598963]);
	c.bench_function("u256_not", move |b| b.iter(|| black_box(!one)));
}

fn u512_not(c: &mut Criterion) {
	let one = U512([
		8326634216714383706,
		15837136097609390493,
		13004317189126203332,
		7031796866963419685,
		12767554894655550452,
		16333049135534778834,
		140317443000293558,
		598963,
	]);
	c.bench_function("u512_not", move |b| b.iter(|| black_box(!one)));
}

fn u256_shl(c: &mut Criterion) {
	let one = U256([12767554894655550452, 16333049135534778834, 140317443000293558, 598963]);
	c.bench_function("u256_shl", move |b| b.iter(|| black_box(one << 128)));
}

fn u512_shl(c: &mut Criterion) {
	let one = U512([
		8326634216714383706,
		15837136097609390493,
		13004317189126203332,
		7031796866963419685,
		12767554894655550452,
		16333049135534778834,
		140317443000293558,
		598963,
	]);
	c.bench_function("u512_shl", move |b| b.iter(|| black_box(one >> 128)));
}

fn u256_shr(c: &mut Criterion) {
	let one = U256([12767554894655550452, 16333049135534778834, 140317443000293558, 598963]);
	c.bench_function("u256_shr", move |b| b.iter(|| black_box(one >> 128)));
}

fn u512_shr(c: &mut Criterion) {
	let one = U512([
		8326634216714383706,
		15837136097609390493,
		13004317189126203332,
		7031796866963419685,
		12767554894655550452,
		16333049135534778834,
		140317443000293558,
		598963,
	]);
	c.bench_function("u512_shr", move |b| b.iter(|| black_box(one >> 128)));
}

fn u256_ord(c: &mut Criterion) {
	let one = U256([12767554894655550452, 16333049135534778834, 140317443000293558, 598963]);
	let two = U256([2096410819092764509, 8483673822214032535, 36306297304129857, 3453]);
	c.bench_function("u256_ord", move |b| b.iter(|| black_box(one) < black_box(two)));
}

fn u512_ord(c: &mut Criterion) {
	let one = U512([
		8326634216714383706,
		15837136097609390493,
		13004317189126203332,
		7031796866963419685,
		12767554894655550452,
		16333049135534778834,
		140317443000293558,
		598963,
	]);
	let two = U512([
		11707750893627518758,
		17679501210898117940,
		2472932874039724966,
		11177683849610900539,
		2096410819092764509,
		8483673822214032535,
		36306297304129857,
		3453,
	]);
	c.bench_function("u512_ord", move |b| b.iter(|| black_box(one) < black_box(two)));
}

fn u256_from_le(c: &mut Criterion) {
	c.bench_function("u256_from_le", |b| {
		b.iter(|| {
			let raw = black_box([
				1u8, 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
				101, 103, 107, 109, 113, 127,
			]);
			black_box(U256::from_little_endian(&raw[..]))
		})
	});
}

fn u256_from_be(c: &mut Criterion) {
	c.bench_function("u256_from_be", |b| {
		b.iter(|| {
			let raw = black_box([
				1u8, 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
				101, 103, 107, 109, 113, 127,
			]);
			black_box(U256::from_big_endian(&raw[..]))
		})
	});
}

fn from_fixed_array(c: &mut Criterion) {
	let ary512: [u8; 64] = [
		255, 0, 0, 123, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 121, 0, 0, 0, 0, 0, 213, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45, 0, 0, 67, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 123,
	];
	let ary256: [u8; 32] =
		[255, 0, 0, 123, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 121, 0, 0, 0, 0, 0, 213, 0, 0, 0, 0, 0, 0];
	c.bench_function("from_fixed_array", move |b| {
		b.iter(|| {
			let _: U512 = black_box(black_box(ary512).into());
			let _: U256 = black_box(black_box(ary256).into());
		})
	});
}

fn from_str(c: &mut Criterion) {
	c.bench_function("from_str", move |b| {
		b.iter(|| {
			black_box(U512::from_str(black_box("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")).unwrap());
			black_box(U512::from_str(black_box("0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")).unwrap());
			black_box(U512::from_str(black_box("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")).unwrap());
		})
	});
}
