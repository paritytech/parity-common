// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

mod keypair;
mod random;
mod signature;
mod secret;
mod extended;

pub mod ecdh;
pub mod ecies;
pub mod math;
pub mod error;

pub use self::keypair::{KeyPair, public_to_address};
pub use self::math::public_is_valid;
pub use self::random::Random;
pub use self::error::Error;
pub use self::signature::{sign, verify_public, verify_address, recover, Signature};
pub use self::secret::Secret;
pub use self::extended::{ExtendedPublic, ExtendedSecret, ExtendedKeyPair, DerivationError, Derivation};

use ethereum_types::H256;
use lazy_static::lazy_static;

pub use ethereum_types::{Address, Public};
pub type Message = H256;

lazy_static! {
	pub static ref SECP256K1: secp256k1::Secp256k1 = secp256k1::Secp256k1::new();
}

/// Uninstantiatable error type for infallible generators.
#[derive(Debug)]
pub enum Void {}

/// Generates new keypair.
pub trait Generator {
	type Error;

	/// Should be called to generate new keypair.
	fn generate(&mut self) -> Result<KeyPair, Self::Error>;
}
