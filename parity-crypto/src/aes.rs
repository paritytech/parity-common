// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use aes::block_cipher_trait::generic_array::GenericArray;
use aes::{Aes128, Aes256};
use aes_ctr::stream_cipher::{NewStreamCipher, SyncStreamCipher};
use block_modes::{
	block_padding::{Pkcs7, ZeroPadding},
	BlockMode, Cbc, Ecb,
};

use crate::error::SymmError;

/// One time encoder/decoder for Ecb mode Aes256 with zero padding
pub struct AesEcb256(Ecb<Aes256, ZeroPadding>);

impl AesEcb256 {
	/// New encoder/decoder, no iv for ecb
	pub fn new(key: &[u8]) -> Result<Self, SymmError> {
		Ok(AesEcb256(Ecb::new_var(key, &[])?))
	}

	/// Encrypt data in place without padding. The data length must be a multiple
	/// of the block size.
	pub fn encrypt(self, content: &mut [u8]) -> Result<(), SymmError> {
		let len = content.len();
		self.0.encrypt(content, len)?;
		Ok(())
	}

	/// Decrypt data in place without padding. The data length must be a multiple
	/// of the block size.
	pub fn decrypt(self, content: &mut [u8]) -> Result<(), SymmError> {
		self.0.decrypt(content)?;
		Ok(())
	}
}

/// Reusable encoder/decoder for Aes256 in Ctr mode and no padding
pub struct AesCtr256(aes_ctr::Aes256Ctr);

impl AesCtr256 {
	/// New encoder/decoder
	pub fn new(key: &[u8], iv: &[u8]) -> Result<Self, SymmError> {
		Ok(AesCtr256(aes_ctr::Aes256Ctr::new(GenericArray::from_slice(key), GenericArray::from_slice(iv))))
	}

	/// In place encrypt a content without padding, the content length must be a multiple
	/// of the block size.
	pub fn encrypt(&mut self, content: &mut [u8]) -> Result<(), SymmError> {
		self.0.try_apply_keystream(content)?;
		Ok(())
	}

	/// In place decrypt a content without padding, the content length must be a multiple
	/// of the block size.
	pub fn decrypt(&mut self, content: &mut [u8]) -> Result<(), SymmError> {
		self.0.try_apply_keystream(content)?;
		Ok(())
	}
}

/// Encrypt a message (CTR mode).
///
/// Key (`k`) length and initialisation vector (`iv`) length have to be 16 bytes each.
/// An error is returned if the input lengths are invalid.
/// If possible prefer `inplace_encrypt_128_ctr` to avoid a slice copy.
pub fn encrypt_128_ctr(k: &[u8], iv: &[u8], plain: &[u8], dest: &mut [u8]) -> Result<(), SymmError> {
	let mut encryptor = aes_ctr::Aes128Ctr::new(GenericArray::from_slice(k), GenericArray::from_slice(iv));
	&mut dest[..plain.len()].copy_from_slice(plain);
	encryptor.try_apply_keystream(dest)?;
	Ok(())
}

/// Encrypt a message (CTR mode).
///
/// Key (`k`) length and initialisation vector (`iv`) length have to be 16 bytes each.
/// An error is returned if the input lengths are invalid.
pub fn inplace_encrypt_128_ctr(k: &[u8], iv: &[u8], data: &mut [u8]) -> Result<(), SymmError> {
	let mut encryptor = aes_ctr::Aes128Ctr::new(GenericArray::from_slice(k), GenericArray::from_slice(iv));
	encryptor.try_apply_keystream(data)?;
	Ok(())
}

/// Decrypt a message (CTR mode).
///
/// Key (`k`) length and initialisation vector (`iv`) length have to be 16 bytes each.
/// An error is returned if the input lengths are invalid.
/// If possible prefer `inplace_decrypt_128_ctr` instead.
pub fn decrypt_128_ctr(k: &[u8], iv: &[u8], encrypted: &[u8], dest: &mut [u8]) -> Result<(), SymmError> {
	let mut encryptor = aes_ctr::Aes128Ctr::new(GenericArray::from_slice(k), GenericArray::from_slice(iv));

	&mut dest[..encrypted.len()].copy_from_slice(encrypted);
	encryptor.try_apply_keystream(dest)?;
	Ok(())
}

/// Decrypt a message (CTR mode).
///
/// Key (`k`) length and initialisation vector (`iv`) length have to be 16 bytes each.
/// An error is returned if the input lengths are invalid.
pub fn inplace_decrypt_128_ctr(k: &[u8], iv: &[u8], data: &mut [u8]) -> Result<(), SymmError> {
	let mut encryptor = aes_ctr::Aes128Ctr::new(GenericArray::from_slice(k), GenericArray::from_slice(iv));

	encryptor.try_apply_keystream(data)?;
	Ok(())
}

/// Decrypt a message (CBC mode).
///
/// Key (`k`) length and initialisation vector (`iv`) length have to be 16 bytes each.
/// An error is returned if the input lengths are invalid.
pub fn decrypt_128_cbc(k: &[u8], iv: &[u8], encrypted: &[u8], dest: &mut [u8]) -> Result<usize, SymmError> {
	let encryptor = Cbc::<Aes128, Pkcs7>::new_var(k, iv)?;
	&mut dest[..encrypted.len()].copy_from_slice(encrypted);
	let unpad_length = { encryptor.decrypt(&mut dest[..encrypted.len()])?.len() };
	Ok(unpad_length)
}

#[cfg(test)]
mod tests {

	use super::*;

	// only use for test could be expose in the future
	fn encrypt_128_cbc(k: &[u8], iv: &[u8], plain: &[u8], dest: &mut [u8]) -> Result<(), SymmError> {
		let encryptor = Cbc::<Aes128, Pkcs7>::new_var(k, iv)?;
		&mut dest[..plain.len()].copy_from_slice(plain);
		encryptor.encrypt(dest, plain.len())?;
		Ok(())
	}

	#[test]
	pub fn test_aes_short() -> Result<(), SymmError> {
		let key = [
			97, 110, 121, 99, 111, 110, 116, 101, 110, 116, 116, 111, 114, 101, 97, 99, 104, 49, 50, 56, 98, 105, 116,
			115, 105, 122, 101, 10,
		];
		let salt = [
			109, 121, 115, 97, 108, 116, 115, 104, 111, 117, 108, 100, 102, 105, 108, 108, 115, 111, 109, 109, 101, 98,
			121, 116, 101, 108, 101, 110, 103, 116, 104, 10,
		];
		let content = [
			83, 111, 109, 101, 32, 99, 111, 110, 116, 101, 110, 116, 32, 116, 111, 32, 116, 101, 115, 116, 32, 97, 101,
			115, 44, 10, 110, 111, 116, 32, 116, 111, 32, 109, 117, 99, 104, 32, 44, 32, 111, 110, 108, 121, 32, 118,
			101, 114, 121, 32, 98, 97, 115, 105, 99, 32, 116, 101, 115, 116, 32, 116, 111, 32, 97, 118, 111, 105, 100,
			32, 111, 98, 118, 105, 111, 117, 115, 32, 114, 101, 103, 114, 101, 115, 115, 105, 111, 110, 32, 119, 104,
			101, 110, 32, 115, 119, 105, 116, 99, 104, 105, 110, 103, 32, 108, 105, 98, 115, 46, 10,
		];
		let ctr_enc = [
			65, 55, 246, 75, 24, 117, 30, 233, 218, 139, 91, 251, 251, 179, 171, 69, 60, 244, 249, 44, 238, 60, 10, 66,
			71, 10, 199, 111, 54, 24, 124, 223, 153, 250, 159, 154, 164, 109, 232, 82, 20, 199, 182, 40, 174, 104, 64,
			203, 236, 94, 222, 184, 117, 54, 234, 189, 253, 122, 135, 121, 100, 44, 227, 241, 123, 120, 110, 188, 109,
			148, 112, 160, 131, 205, 116, 104, 232, 8, 22, 170, 80, 231, 155, 246, 255, 115, 101, 5, 234, 104, 220,
			199, 192, 166, 181, 156, 113, 255, 187, 51, 38, 128, 75, 29, 237, 178, 205, 98, 101, 110,
		];
		let cbc_enc = [
			167, 248, 5, 90, 11, 140, 215, 138, 165, 125, 137, 76, 47, 243, 191, 48, 183, 247, 109, 86, 24, 45, 81,
			215, 0, 51, 221, 185, 131, 97, 234, 189, 244, 255, 107, 210, 70, 60, 41, 221, 43, 137, 185, 166, 42, 65,
			18, 200, 151, 233, 255, 192, 109, 25, 105, 115, 161, 209, 126, 235, 99, 192, 241, 241, 19, 249, 87, 244,
			28, 146, 186, 189, 108, 9, 243, 132, 4, 105, 53, 162, 8, 235, 84, 107, 213, 59, 158, 113, 227, 120, 162,
			50, 237, 123, 70, 187, 83, 73, 146, 13, 44, 191, 53, 4, 125, 207, 176, 45, 8, 153, 175, 198,
		];
		let mut dest = vec![0; 110];
		let mut dest_padded = vec![0; 112];
		let mut dest_padded2 = vec![0; 128]; // TODO RustLib need an extra 16bytes in dest : looks extra buggy but function is not currently use (keep it private for now)
		encrypt_128_cbc(&key[..16], &salt[..16], &content, &mut dest_padded2)?;
		assert!(&dest_padded2[..112] == &cbc_enc[..]);
		encrypt_128_ctr(&key[..16], &salt[..16], &content, &mut dest)?;
		assert!(&dest[..] == &ctr_enc[..]);
		let mut content_data = content.to_vec();
		inplace_encrypt_128_ctr(&key[..16], &salt[..16], &mut content_data[..])?;
		assert!(&content_data[..] == &ctr_enc[..]);
		decrypt_128_ctr(&key[..16], &salt[..16], &ctr_enc[..], &mut dest)?;
		assert!(&dest[..] == &content[..]);
		let mut content_data = ctr_enc.to_vec();
		inplace_decrypt_128_ctr(&key[..16], &salt[..16], &mut content_data[..])?;
		assert!(&content_data[..] == &content[..]);
		let l = decrypt_128_cbc(&key[..16], &salt[..16], &cbc_enc[..], &mut dest_padded)?;
		assert!(&dest_padded[..l] == &content[..]);
		Ok(())
	}
}
