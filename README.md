# bigint

[![Build Status](https://travis-ci.org/paritytech/bigint.svg?branch=master)](https://travis-ci.org/paritytech/bigint)

Fixed-sized integers arithmetic

```rust
extern crate bigint;
use bigint::{U256, Uint};

fn main() {
		let mut val: U256 = 1023.into();
		for _ in 0..200 { val = val * 2.into() }
		assert_eq!(
			&format!("{}", val), 
			"1643897619276947051879427220465009342380213662639797070513307648"
		);
}
