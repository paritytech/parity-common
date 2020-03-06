// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::parity_crypto::publickey::Generator;
use criterion::{criterion_group, criterion_main, Bencher, Criterion};

criterion_group!(benches, input_len, ecdh_agree,);

criterion_main!(benches);

/// general benches for multiple input size
fn input_len(c: &mut Criterion) {
	c.bench_function_over_inputs(
		"ripemd",
		|b: &mut Bencher, size: &usize| {
			let data = vec![0u8; *size];
			b.iter(|| parity_crypto::digest::ripemd160(&data[..]));
		},
		vec![100, 500, 1_000, 10_000, 100_000],
	);

	c.bench_function_over_inputs(
		"aes_ctr",
		|b: &mut Bencher, size: &usize| {
			let data = vec![0u8; *size];
			let mut dest = vec![0; *size];
			let k = [0; 16];
			let iv = [0; 16];

			b.iter(|| {
				parity_crypto::aes::encrypt_128_ctr(&k[..], &iv[..], &data[..], &mut dest[..]).unwrap();
				// same as encrypt but add it just in case
				parity_crypto::aes::decrypt_128_ctr(&k[..], &iv[..], &data[..], &mut dest[..]).unwrap();
			});
		},
		vec![100, 500, 1_000, 10_000, 100_000],
	);
}

fn ecdh_agree(c: &mut Criterion) {
	let keypair = parity_crypto::publickey::Random.generate().unwrap();
	let public = keypair.public().clone();
	let secret = keypair.secret().clone();

	c.bench_function("ecdh_agree", move |b| b.iter(|| parity_crypto::publickey::ecdh::agree(&secret, &public)));
}
