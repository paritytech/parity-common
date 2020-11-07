// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Random key pair generator. Relies on the secp256k1 C-library to generate random data.

use super::{Generator, KeyPair};
use secp256k1::SECP256K1;

/// Randomly generates new keypair, instantiating the RNG each time.
pub struct Random;

impl Generator for Random {
	fn generate(&mut self) -> KeyPair {
		let (sec, publ) = SECP256K1.generate_keypair(&mut secp256k1::rand::thread_rng());
		KeyPair::from_keypair(sec, publ)
	}
}
