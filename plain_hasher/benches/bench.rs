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

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use criterion::{criterion_group, criterion_main, Criterion};
use plain_hasher::PlainHasher;

fn bench_write_hasher(c: &mut Criterion) {
	c.bench_function("write_plain_hasher", |b| b.iter(|| {
		(0..100u8).fold(PlainHasher::default(), |mut old, new| {
			let bb = [new; 32];
			old.write(&bb);
			old
		});
	}));
	c.bench_function("write_default_hasher", |b| b.iter(|| {
		(0..100u8).fold(DefaultHasher::default(), |mut old, new| {
			let bb = [new; 32];
			old.write(&bb);
			old
		});
	}));
}

criterion_group!(benches, bench_write_hasher);
criterion_main!(benches);
