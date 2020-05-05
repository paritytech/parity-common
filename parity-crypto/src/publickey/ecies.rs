// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Functions for ECIES scheme encryption and decryption

use super::{ecdh, Error, Generator, Public, Random, Secret};
use crate::{aes, digest, hmac, is_equal};
use ethereum_types::H128;

const ENC_VERSION: u8 = 0x04;

/// Encrypt a message with a public key, writing an HMAC covering both
/// the plaintext and authenticated data.
///
/// Authenticated data may be empty.
pub fn encrypt(public: &Public, auth_data: &[u8], plain: &[u8]) -> Result<Vec<u8>, Error> {
	let r = Random.generate();
	let z = ecdh::agree(r.secret(), public)?;
	let mut key = [0u8; 32];
	kdf(&z, &[0u8; 0], &mut key);

	let ekey = &key[0..16];
	let mkey = hmac::SigKey::sha256(&digest::sha256(&key[16..32]));

	let mut msg = vec![0u8; 1 + 64 + 16 + plain.len() + 32];
	msg[0] = ENC_VERSION;
	{
		let result_msg = &mut msg[1..];
		result_msg[0..64].copy_from_slice(r.public().as_bytes());
		let iv = H128::random();
		result_msg[64..80].copy_from_slice(iv.as_bytes());
		{
			let cipher = &mut result_msg[(64 + 16)..(64 + 16 + plain.len())];
			aes::encrypt_128_ctr(ekey, iv.as_bytes(), plain, cipher)?;
		}
		let mut hmac = hmac::Signer::with(&mkey);
		{
			let cipher_iv = &result_msg[64..(64 + 16 + plain.len())];
			hmac.update(cipher_iv);
		}
		hmac.update(auth_data);
		let sig = hmac.sign();
		result_msg[(64 + 16 + plain.len())..].copy_from_slice(&sig);
	}
	Ok(msg)
}

/// Decrypt a message with a secret key, checking HMAC for ciphertext
/// and authenticated data validity.
pub fn decrypt(secret: &Secret, auth_data: &[u8], encrypted: &[u8]) -> Result<Vec<u8>, Error> {
	const META_LEN: usize = 1 + 64 + 16 + 32;
	let enc_version = encrypted[0];
	if encrypted.len() < META_LEN || enc_version < 2 || enc_version > 4 {
		return Err(Error::InvalidMessage);
	}

	let e = &encrypted[1..];
	let p = Public::from_slice(&e[0..64]);
	let z = ecdh::agree(secret, &p)?;
	let mut key = [0u8; 32];
	kdf(&z, &[0u8; 0], &mut key);

	let ekey = &key[0..16];
	let mkey = hmac::SigKey::sha256(&digest::sha256(&key[16..32]));

	let cipher_text_len = encrypted.len() - META_LEN;
	let cipher_with_iv = &e[64..(64 + 16 + cipher_text_len)];
	let cipher_iv = &cipher_with_iv[0..16];
	let cipher_no_iv = &cipher_with_iv[16..];
	let msg_mac = &e[(64 + 16 + cipher_text_len)..];

	// Verify tag
	let mut hmac = hmac::Signer::with(&mkey);
	hmac.update(cipher_with_iv);
	hmac.update(auth_data);
	let mac = hmac.sign();

	if !is_equal(&mac.as_ref()[..], msg_mac) {
		return Err(Error::InvalidMessage);
	}

	let mut msg = vec![0u8; cipher_text_len];
	aes::decrypt_128_ctr(ekey, cipher_iv, cipher_no_iv, &mut msg[..])?;
	Ok(msg)
}

fn kdf(secret: &Secret, s1: &[u8], dest: &mut [u8]) {
	// SEC/ISO/Shoup specify counter size SHOULD be equivalent
	// to size of hash output, however, it also notes that
	// the 4 bytes is okay. NIST specifies 4 bytes.
	let mut ctr = 1u32;
	let mut written = 0usize;
	while written < dest.len() {
		let mut hasher = digest::Hasher::sha256();
		let ctrs = [(ctr >> 24) as u8, (ctr >> 16) as u8, (ctr >> 8) as u8, ctr as u8];
		hasher.update(&ctrs);
		hasher.update(secret.as_bytes());
		hasher.update(s1);
		let d = hasher.finish();
		&mut dest[written..(written + 32)].copy_from_slice(&d);
		written += 32;
		ctr += 1;
	}
}

#[cfg(test)]
mod tests {
	use super::super::{ecies, Generator, Random};

	#[test]
	fn ecies_shared() {
		let kp = Random.generate();
		let message = b"So many books, so little time";

		let shared = b"shared";
		let wrong_shared = b"incorrect";
		let encrypted = ecies::encrypt(kp.public(), shared, message).unwrap();
		assert!(encrypted[..] != message[..]);
		assert_eq!(encrypted[0], 0x04);

		assert!(ecies::decrypt(kp.secret(), wrong_shared, &encrypted).is_err());
		let decrypted = ecies::decrypt(kp.secret(), shared, &encrypted).unwrap();
		assert_eq!(decrypted[..message.len()], message[..]);
	}
}
