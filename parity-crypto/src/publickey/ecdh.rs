// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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
	let shared = ecdh::SharedSecret::new_with_hash(&publ, &sec, |x, _| x.into());

	Secret::import_key(&shared[0..32]).map_err(|_| Error::Secp(secp256k1::Error::InvalidSecretKey))
}

#[cfg(test)]
mod tests {
	use super::{agree, Public, Secret};
	use std::str::FromStr;

	#[test]
	fn test_agree() {
		// Just some random values for secret/public to check we agree with previous implementation.
		let secret =
			Secret::copy_from_str(&"01a400760945613ff6a46383b250bf27493bfe679f05274916182776f09b28f1").unwrap();
		let public= Public::from_str("e37f3cbb0d0601dc930b8d8aa56910dd5629f2a0979cc742418960573efc5c0ff96bc87f104337d8c6ab37e597d4f9ffbd57302bc98a825519f691b378ce13f5").unwrap();
		let shared = agree(&secret, &public);

		assert!(shared.is_ok());
		assert_eq!(shared.unwrap().to_hex(), "28ab6fad6afd854ff27162e0006c3f6bd2daafc0816c85b5dfb05dbb865fa6ac",);
	}
}
