// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Submodule of crypto utils for working with public key crypto primitives
//! If you are looking for git history please refer to the `ethkey` crate in the `parity-ethereum` repository.

mod ecdsa_signature;
mod extended_keys;
mod keypair;
mod keypair_generator;
mod secret_key;

pub mod ec_math_utils;
pub mod ecdh;
pub mod ecies;
pub mod error;

pub use self::{
	ecdsa_signature::{recover, sign, verify_address, verify_public, Signature},
	error::Error,
	extended_keys::{Derivation, DerivationError, ExtendedKeyPair, ExtendedPublic, ExtendedSecret},
	keypair::{public_to_address, KeyPair},
	keypair_generator::Random,
	secret_key::{Secret, ZeroizeSecretKey},
};

use ethereum_types::H256;

pub use ethereum_types::{Address, Public};
pub type Message = H256;

/// The number -1 encoded as a secret key
const MINUS_ONE_KEY: &'static [u8] = &[
	0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0xba, 0xae, 0xdc,
	0xe6, 0xaf, 0x48, 0xa0, 0x3b, 0xbf, 0xd2, 0x5e, 0x8c, 0xd0, 0x36, 0x41, 0x40,
];

/// Generates new keypair.
pub trait Generator {
	/// Should be called to generate new keypair.
	fn generate(&mut self) -> KeyPair;
}
