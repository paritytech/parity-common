# Ethereum primitives

[![Build Status](https://travis-ci.org/paritytech/primitives.svg?branch=master)](https://travis-ci.org/paritytech/primitives)

Fixed-sized integer arithmetic (ethereum-types) and bloom filter (ethbloom)

To add this crate to your project, add the following in `Cargo.toml`

```toml
[dependencies]
ethereum-types = "0.4"
ethbloom = "0.5"
```

A basic example how to use this crate:

```rust
extern crate ethereum_types;
extern crate ethbloom;

use ethereum_types::U256;
use ethbloom::{Bloom, Input};

fn main() {
	let mut val: U256 = 1023.into();
	for _ in 0..200 { val = val * 2u32 }
	assert_eq!(
		&format!("{}", val),
		"1643897619276947051879427220465009342380213662639797070513307648"
	);

	let address = [0_u8; 32];
	let mut my_bloom = Bloom::default();
	assert!(!my_bloom.contains_input(Input::Raw(&address)));
	my_bloom.accrue(Input::Raw(&address));
}

```

### `no_std` crates

This crate has a feature, `std`, that is enabled by default. To use this crate
in a `no_std` context, add the following to your `Cargo.toml`:

```toml
[dependencies]
ethereum-types = { version = "0.4", default-features = false }
ethbloom = { version = "0.5", default-features = false }
```
