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

//! ECDH key agreement scheme implemented as a free function.

use super::{Error, Public, Secret, SECP256K1};
use secp256k1::{self, ecdh, key};

/// Agree on a shared secret
pub fn agree(secret: &Secret, public: &Public) -> Result<Secret, Error> {
	let context = &SECP256K1;
	let pdata = {
		let mut temp = [4u8; 65];
		(&mut temp[1..65]).copy_from_slice(&public[0..64]);
		temp
	};

	let publ = key::PublicKey::from_slice(&pdata)?;
	let sec = key::SecretKey::from_slice(secret.as_bytes())?;
	let shared = ecdh::SharedSecret::new_raw(context, &publ, &sec);

	Secret::import_key(&shared[0..32]).map_err(|_| Error::Secp(secp256k1::Error::InvalidSecretKey))
}
