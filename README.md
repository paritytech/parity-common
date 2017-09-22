# bigint

[![Build Status](https://travis-ci.org/paritytech/bigint.svg?branch=master)](https://travis-ci.org/paritytech/bigint)

[API Documentation](https://docs.rs/bigint/)

Fixed-sized integers arithmetic

Add a dependency

[dependencies]
bigint = "4"

Little example

```rust
extern crate bigint;
use bigint::U256;

fn main() {
	let mut val: U256 = 1023.into();
	for _ in 0..200 { val = val * 2.into() }
	assert_eq!(
		&format!("{}", val), 
		"1643897619276947051879427220465009342380213662639797070513307648"
	);
}
```

### `no_std` crates

This crate has a feature, `std`, that is enabled by default. To use this crate
in a `no_std` context, add the following to your `Cargo.toml`:

```toml
[dependencies]
bigint = { version = "4", default-features = false }
```
