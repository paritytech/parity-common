// Copyright 2015-2017 Parity Technologies
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

#[macro_use]
extern crate criterion;
extern crate core;
#[macro_use]
extern crate uint;
extern crate num_bigint;
extern crate rug;

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
use rug::{Integer, integer::Order};
use std::str::FromStr;

criterion_group!(
	bigint,
	u256_add,
	u256_sub,
	u256_mul,
	u256_mul_small,
	u256_mul_full,
	u256_div,
	u256_rem,
	u256_rem_small,
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
	u512_mul_small,
	u512_div,
	u512_rem,
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
	from_fixed_array,
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
	let mut bytes = [0u8; 32];
	x.to_big_endian(&mut bytes);
	Integer::from_digits(&bytes, Order::MsfBe)
}

fn from_gmp(x: Integer) -> U512 {
	let digits = x.to_digits(Order::MsfBe);
	U512::from_big_endian(&digits)
}

fn u256_add(c: &mut Criterion) {
	c.bench_function("u256_add", |b| {
		b.iter(|| {
			let n = 10000;
			let zero = U256::zero();
			(0..n).fold(zero, |old, new| old.overflowing_add(U256::from(new)).0)
		})
	});
}

fn u256_sub(b: &mut Criterion) {
	b.bench_function("u256_sub", |b| bench_u256_sub(b));
}

fn bench_u256_sub(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		let max = black_box(U256::max_value());
		(0..n).fold(max, |old, new| old.overflowing_sub(U256::from(black_box(new))).0)
	});
}

fn u256_mul(b: &mut Criterion) {
	b.bench_function("u256_mul", |b| bench_u256_mul(b));
}

fn bench_u256_mul(b: &mut Bencher) {
	b.iter(|| {
		(1..10000).fold(black_box(U256::one()), |old, new| {
			old.overflowing_mul(U256::from(black_box(new | 1))).0
		})
	});
}

fn u256_mul_small(b: &mut Criterion) {
	b.bench_function("u256_mul_small", |b| bench_u256_mul_small(b));
}

fn bench_u256_mul_small(b: &mut Bencher) {
	b.iter(|| {
		(1..77)
			.fold(black_box(U256::one()), |old, _| old.overflowing_mul(U256::from(black_box(10))).0)
	});
}

fn u256_mul_full(b: &mut Criterion) {
	b.bench_function("u256_mul_full", |b| bench_u256_mul_full(b));
}

fn bench_u256_mul_full(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		let one = black_box(U256::one());
		(1..n).map(|n| n | 1).fold(one, |old, new| {
			let new = black_box(new);
			let U512(ref u512words) = old.full_mul(U256([new, new, new, new]));
			U256([u512words[0], u512words[2], u512words[2], u512words[3]])
		})
	});
}

fn u256_div(b: &mut Criterion) {
	b.bench_function("u256_div", |b| bench_u256_div(b));
}

fn bench_u256_div(b: &mut Bencher) {
	let one = U256([12767554894655550452, 16333049135534778834, 140317443000293558, 598963]);
	let two = U256([2096410819092764509, 8483673822214032535, 36306297304129857, 3453]);
	b.iter(|| {
		black_box(one / two);
	});
}

fn u256_rem(b: &mut Criterion) {
	b.bench_function("u256_rem", |b| bench_u256_rem(b));
}

fn bench_u256_rem(b: &mut Bencher) {
	let one = U256([12767554894655550452, 16333049135534778834, 140317443000293558, 598963]);
	let two = U256([2096410819092764509, 8483673822214032535, 36306297304129857, 3453]);
	b.iter(|| {
		black_box(one % two);
	});
}

fn u256_rem_small(b: &mut Criterion) {
	b.bench_function("u256_rem_small", |b| bench_u256_rem_small(b));
}

fn bench_u256_rem_small(b: &mut Bencher) {
	let x =
		U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
	let y =
		U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
	let z = U256::from(1u64);
	b.iter(|| {
		let w = black_box(x.overflowing_mul(y)).0;
		black_box(w % z);
	});
}

fn u512_add(b: &mut Criterion) {
	b.bench_function("u512_add", |b| bench_u512_add(b));
}

fn bench_u512_add(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		let zero = black_box(U512::zero());
		(0..n).fold(zero, |old, new| {
			let new = black_box(new);
			old.overflowing_add(U512([new, new, new, new, new, new, new, new])).0
		})
	});
}

fn u512_sub(b: &mut Criterion) {
	b.bench_function("u512_sub", |b| bench_u512_sub(b));
}

fn bench_u512_sub(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		let max = black_box(U512::max_value());
		(0..n).fold(max, |old, new| {
			let new = black_box(new);
			let p = new % 2;
			old.overflowing_sub(U512([p, p, p, p, p, p, p, new])).0
		})
	});
}

fn u512_mul(b: &mut Criterion) {
	b.bench_function("u512_mul", |b| bench_u512_mul(b));
}

fn bench_u512_mul(b: &mut Bencher) {
	let one =
		U512::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
	let two =
		U512::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
	b.iter(|| black_box(one).overflowing_mul(black_box(two)).0);
}

fn u512_mul_small(b: &mut Criterion) {
	b.bench_function("u512_mul_small", |b| bench_u512_mul_small(b));
}

fn bench_u512_mul_small(b: &mut Bencher) {
	b.iter(|| {
		(1..153)
			.fold(black_box(U512::one()), |old, _| old.overflowing_mul(U512::from(black_box(10))).0)
	});
}

fn u512_div(b: &mut Criterion) {
	b.bench_function("u512_div", |b| bench_u512_div(b));
}

fn bench_u512_div(b: &mut Bencher) {
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
	b.iter(|| {
		black_box(one / two);
	});
}

fn u512_rem(b: &mut Criterion) {
	b.bench_function("u512_rem", |b| bench_u512_rem(b));
}

fn bench_u512_rem(b: &mut Bencher) {
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
	b.iter(|| {
		black_box(one % two);
	});
}

fn conversions(b: &mut Criterion) {
	b.bench(
		"conversions biguint vs gmp",
		ParameterizedBenchmark::new("BigUint", |b, i| bench_convert_to_biguit(b, *i), vec![0, 42, u64::max_value()])
			.with_function("gmp", |b, i| bench_convert_to_gmp(b, *i))
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

fn mulmod_u512_vs_biguint_vs_gmp(b: &mut Criterion) {
	let mods = vec![1u64, 42, 10_000_001, u64::max_value()];
	b.bench(
		"mulmod u512 vs biguint vs gmp",
		ParameterizedBenchmark::new("u512", |b, i| bench_u512_mulmod(b, *i), mods)
			.with_function("BigUint", |b, i| bench_biguint_mulmod(b, *i))
			.with_function("gmp", |b, i| bench_gmp_mulmod(b, *i)),
	);
}

fn bench_biguint_mulmod(b: &mut Bencher, i: u64) {
	let x =
		U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
	let y =
		U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
	let z = U256::from(i);
	b.iter(|| {
		let w = to_biguint(x) * to_biguint(y);
		from_biguint(w % to_biguint(z));
	});
}

fn bench_gmp_mulmod(b: &mut Bencher, i: u64) {
	let x =
		U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
	let y =
		U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
	let z = U256::from(i);
	b.iter(|| {
		let w = to_gmp(x) * to_gmp(y);
		from_gmp(w % to_gmp(z));
	});
}

fn bench_u512_mulmod(b: &mut Bencher, i: u64) {
	let x =
		U512::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
	let y =
		U512::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap();
	let z = U512::from(i);
	b.iter(|| {
		let w = x.overflowing_mul(y).0;
		black_box(w % z)
	});
}

// NOTE: uses native `u128` and does not measure this crates performance,
// but might be interesting as a comparison.

fn u128_mul(b: &mut Criterion) {
	b.bench_function("u128_mul", |b| bench_u128_mul(b));
}

fn bench_u128_mul(b: &mut Bencher) {
	b.iter(|| {
		let n = black_box(10000);
		(1..n).fold(12345u128, |old, new| old.overflowing_mul(u128::from(new | 1u32)).0)
	});
}

fn u256_bit_and(b: &mut Criterion) {
	b.bench_function("u256_bit_and", |b| bench_u256_bit_and(b));
}

fn bench_u256_bit_and(b: &mut Bencher) {
	let one = U256([12767554894655550452, 16333049135534778834, 140317443000293558, 598963]);
	let two = U256([2096410819092764509, 8483673822214032535, 36306297304129857, 3453]);
	b.iter(|| black_box(one) & black_box(two));
}

fn u512_bit_and(b: &mut Criterion) {
	b.bench_function("u512_bit_and", |b| bench_u512_bit_and(b));
}

fn bench_u512_bit_and(b: &mut Bencher) {
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
	b.iter(|| black_box(one) & black_box(two));
}

fn u256_bit_xor(b: &mut Criterion) {
	b.bench_function("u256_bit_xor", |b| bench_u256_bit_xor(b));
}

fn bench_u256_bit_xor(b: &mut Bencher) {
	let one = U256([12767554894655550452, 16333049135534778834, 140317443000293558, 598963]);
	let two = U256([2096410819092764509, 8483673822214032535, 36306297304129857, 3453]);
	b.iter(|| black_box(one) ^ black_box(two));
}

fn u512_bit_xor(b: &mut Criterion) {
	b.bench_function("u512_bit_xor", |b| bench_u512_bit_xor(b));
}

fn bench_u512_bit_xor(b: &mut Bencher) {
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
	b.iter(|| black_box(one) ^ black_box(two));
}

fn u256_bit_or(b: &mut Criterion) {
	b.bench_function("u256_bit_or", |b| bench_u256_bit_or(b));
}

fn bench_u256_bit_or(b: &mut Bencher) {
	let one = U256([12767554894655550452, 16333049135534778834, 140317443000293558, 598963]);
	let two = U256([2096410819092764509, 8483673822214032535, 36306297304129857, 3453]);
	b.iter(|| black_box(one) | black_box(two));
}

fn u512_bit_or(b: &mut Criterion) {
	b.bench_function("u512_bit_or", |b| bench_u512_bit_or(b));
}

fn bench_u512_bit_or(b: &mut Bencher) {
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
	b.iter(|| black_box(one) | black_box(two));
}

fn u256_not(b: &mut Criterion) {
	b.bench_function("u256_not", |b| bench_u256_not(b));
}

fn bench_u256_not(b: &mut Bencher) {
	let one = U256([12767554894655550452, 16333049135534778834, 140317443000293558, 598963]);
	b.iter(|| !black_box(one));
}

fn u512_not(b: &mut Criterion) {
	b.bench_function("u512_not", |b| bench_u512_not(b));
}

fn bench_u512_not(b: &mut Bencher) {
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
	b.iter(|| !black_box(one));
}

fn u256_shl(b: &mut Criterion) {
	b.bench_function("u256_shl", |b| bench_u256_shl(b));
}

fn bench_u256_shl(b: &mut Bencher) {
	let one = U256([12767554894655550452, 16333049135534778834, 140317443000293558, 598963]);
	b.iter(|| black_box(one) << 128);
}

fn u512_shl(b: &mut Criterion) {
	b.bench_function("u512_shl", |b| bench_u512_shl(b));
}

fn bench_u512_shl(b: &mut Bencher) {
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
	b.iter(|| black_box(one) << 128);
}

fn u256_shr(b: &mut Criterion) {
	b.bench_function("u256_shr", |b| bench_u256_shr(b));
}

fn bench_u256_shr(b: &mut Bencher) {
	let one = U256([12767554894655550452, 16333049135534778834, 140317443000293558, 598963]);
	b.iter(|| black_box(one) >> 128);
}

fn u512_shr(b: &mut Criterion) {
	b.bench_function("u512_shr", |b| bench_u512_shr(b));
}

fn bench_u512_shr(b: &mut Bencher) {
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
	b.iter(|| black_box(one) >> 128);
}

fn u256_ord(b: &mut Criterion) {
	b.bench_function("u256_ord", |b| bench_u256_ord(b));
}

fn bench_u256_ord(b: &mut Bencher) {
	let one = U256([12767554894655550452, 16333049135534778834, 140317443000293558, 598963]);
	let two = U256([2096410819092764509, 8483673822214032535, 36306297304129857, 3453]);
	b.iter(|| black_box(one) < black_box(two));
}

fn u512_ord(b: &mut Criterion) {
	b.bench_function("u512_ord", |b| bench_u512_ord(b));
}

fn bench_u512_ord(b: &mut Bencher) {
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
	b.iter(|| black_box(one) < black_box(two));
}

fn u256_from_le(b: &mut Criterion) {
	b.bench_function("u256_from_le", |b| bench_u256_from_le(b));
}

fn bench_u256_from_le(b: &mut Bencher) {
	b.iter(|| {
		let raw = black_box([
			1u8, 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73,
			79, 83, 89, 97, 101, 103, 107, 109, 113, 127,
		]);
		let _ = U256::from_little_endian(&raw[..]);
	});
}

fn u256_from_be(b: &mut Criterion) {
	b.bench_function("u256_from_be", |b| bench_u256_from_be(b));
}

fn bench_u256_from_be(b: &mut Bencher) {
	b.iter(|| {
		let raw = black_box([
			1u8, 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73,
			79, 83, 89, 97, 101, 103, 107, 109, 113, 127,
		]);
		let _ = U256::from_big_endian(&raw[..]);
	});
}

fn from_fixed_array(b: &mut Criterion) {
	b.bench_function("from_fixed_array", |b| bench_from_fixed_array(b));
}

fn bench_from_fixed_array(b: &mut Bencher) {
	let ary512: [u8; 64] = [
		255, 0, 0, 123, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 121, 0, 0, 0, 0, 0, 213, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45, 0, 0, 67, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 123,
	];
	let ary256: [u8; 32] = [
		255, 0, 0, 123, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 121, 0, 0, 0, 0, 0, 213, 0, 0,
		0, 0, 0, 0,
	];
	b.iter(|| {
		let n = black_box(1000);
		for _i in 0..n {
			let _: U512 = black_box(ary512).into();
			let _: U256 = black_box(ary256).into();
		}
	})
}
