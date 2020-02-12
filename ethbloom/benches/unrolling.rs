// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use criterion::{criterion_group, criterion_main, Criterion};
use crunchy::unroll;
use rand::RngCore;

fn random_data() -> [u8; 256] {
	let mut res = [0u8; 256];
	rand::thread_rng().fill_bytes(&mut res);
	res
}

fn bench_forwards(c: &mut Criterion) {
	c.bench_function("forwards_with_crunchy", |b| {
		let mut data = random_data();
		b.iter(|| {
			let other_data = random_data();
			unroll! {
				for i in 0..255 {
					data[i] |= other_data[i];
				}
			}
		});
	});
	c.bench_function("forwards_without_crunchy", |b| {
		let mut data = random_data();
		b.iter(|| {
			let other_data = random_data();
			for i in 0..255 {
				data[i] |= other_data[i];
			}
		});
	});
}

fn bench_backwards(c: &mut Criterion) {
	c.bench_function("backwards_with_crunchy", |b| {
		let mut data = random_data();
		b.iter(|| {
			let other_data = random_data();
			unroll! {
				for i in 0..255 {
					data[255-i] |= other_data[255-i];
				}
			}
		});
	});
	c.bench_function("backwards_without_crunchy", |b| {
		let mut data = random_data();
		b.iter(|| {
			let other_data = random_data();
			for i in 0..255 {
				data[255 - i] |= other_data[255 - i];
			}
		});
	});
}

criterion_group!(benches, bench_forwards, bench_backwards);
criterion_main!(benches);
