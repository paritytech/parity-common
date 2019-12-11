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

use super::{Error, Public, Secret};
use secp256k1::{self, ecdh, key};

/// Agree on a shared secret
pub fn agree(secret: &Secret, public: &Public) -> Result<Secret, Error> {
	let pdata = {
		let mut temp = [4u8; 65];
		(&mut temp[1..65]).copy_from_slice(&public[0..64]);
		temp
	};

	let publ = key::PublicKey::from_slice(&pdata)?;
	let sec = key::SecretKey::from_slice(secret.as_bytes())?;
	// todo[dvdplm]: could go with new_with_hash_no_panic here (unsafe), but without a benchmark I don't see why.
	let shared = ecdh::SharedSecret::new_with_hash(&publ, &sec, |x, _| {
		x.into()
	})?;

	Secret::import_key(&shared[0..32]).map_err(|_| Error::Secp(secp256k1::Error::InvalidSecretKey))
}

#[cfg(test)]
mod tests {
	use super::{Secret, Public, agree};
	use std::str::FromStr;

	#[test]
	fn test_agree() {
		// Just some random values for secret/public to check we agree with previous implementation.
		let secret = Secret::from_str("01a400760945613ff6a46383b250bf27493bfe679f05274916182776f09b28f1").unwrap();
		let public= Public::from_str("e37f3cbb0d0601dc930b8d8aa56910dd5629f2a0979cc742418960573efc5c0ff96bc87f104337d8c6ab37e597d4f9ffbd57302bc98a825519f691b378ce13f5").unwrap();
		let shared = agree(&secret, &public);

		assert!(shared.is_ok());
		assert_eq!(
			shared.unwrap().to_hex(),
			"28ab6fad6afd854ff27162e0006c3f6bd2daafc0816c85b5dfb05dbb865fa6ac",
		);
	}
}
