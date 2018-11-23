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


//! Benches related to asym crypto, mainly signing and veryfing.

use criterion::{Criterion, Bencher};

use parity_crypto::traits::asym::*;

pub fn secp256k1(c: &mut Criterion) {
	use parity_crypto::secp256k1::Secp256k1;
	asym_bench::<Secp256k1>(c, "secp256k1".to_owned())
}

#[cfg(feature="alt")]
pub fn secp256k1_alt(c: &mut Criterion) {
	use parity_crypto::secp256k1_alt::Secp256k1;
	asym_bench::<Secp256k1>(c, "secp256k1_alt".to_owned())
}



fn asym_bench<A: Asym>(c: &mut Criterion, name: String) {

	c.bench_function(&(name.clone() + "_sign_verify"),
		|b: &mut Bencher| {
			let mut sec_buf = vec![7; A::SECRET_SIZE];
			let message = vec![0;32];
			b.iter(|| {
				let (secret, public) = A::keypair_from_slice(&mut sec_buf).unwrap();
				let signature = secret.sign(&message).unwrap();
				assert!(public.verify(&signature, &message).unwrap());
			});
		}
	);

	c.bench_function(&(name.clone() + "_sign_recover"),
		|b: &mut Bencher| {
			let mut sec_buf = vec![3; A::SECRET_SIZE];
			let message = vec![0;32];
			b.iter(|| {
				let (secret, public) = A::keypair_from_slice(&mut sec_buf).unwrap();
				let signature = secret.sign(&message).unwrap();
				assert!(public == A::recover(&signature, &message).unwrap());
			});
		}
	);

}
