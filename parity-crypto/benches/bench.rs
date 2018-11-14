// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.


extern crate parity_crypto;

#[macro_use]
extern crate criterion;

mod asym;

use criterion::{Criterion, Bencher};

#[cfg(not(feature="alt"))]
criterion_group!(benches, input_len, asym::secp256k1);

#[cfg(feature="alt")]
criterion_group!(benches, input_len, asym::secp256k1, asym::secp256k1_alt);

criterion_main!(benches);

/// general benches for multiple input size
fn input_len(c: &mut Criterion) {

	c.bench_function_over_inputs("ripemd",
		|b: &mut Bencher, size: &usize| {
			let data = vec![0u8; *size];
			b.iter(|| parity_crypto::digest::ripemd160(&data[..]));
		},
		vec![100, 500, 1_000, 10_000, 100_000]
	);

	c.bench_function_over_inputs("aes_ctr",
		|b: &mut Bencher, size: &usize| {
			let data = vec![0u8; *size];
			let mut dest = vec![0; *size];
			let k = [0; 16];
			let iv = [0; 16];
 
			b.iter(||{
				parity_crypto::aes::encrypt_128_ctr(&k[..], &iv[..], &data[..], &mut dest[..]).unwrap();
				// same as encrypt but add it just in case
				parity_crypto::aes::decrypt_128_ctr(&k[..], &iv[..], &data[..], &mut dest[..]).unwrap();
			});
		},
		vec![100, 500, 1_000, 10_000, 100_000]
	);

}
