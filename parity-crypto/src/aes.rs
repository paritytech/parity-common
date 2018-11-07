// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use block_modes::{ BlockMode, BlockModeIv };
use block_modes::block_padding::Pkcs7;
use block_modes::block_padding::ZeroPadding;
use block_modes::{ Cbc, Ecb };
use raes::{ Aes128, Aes256 };
use aes_ctr::{ Aes128Ctr, Aes256Ctr };
use aes_ctr::stream_cipher::{ NewFixStreamCipher, StreamCipherCore };
use error::SymmError;
use raes::block_cipher_trait::generic_array::GenericArray;


/// Reusable encoder/decoder for Ecb mode Aes256 with zero padding
pub struct AesEcb256(Ecb<Aes256, ZeroPadding>);

impl AesEcb256 {

	/// New encoder/decoder, no iv for ecb
	#[inline]
	pub fn new(key: &[u8]) -> Result<Self, SymmError> {
		Ok(AesEcb256(Ecb::new_varkey(key)?))
	}

	#[inline]
	pub fn encrypt(&mut self, content: &mut [u8]) -> Result<(), SymmError> {
		self.0.encrypt_nopad(content)?;
		Ok(())
	}

	#[inline]
	pub fn decrypt(&mut self, content: &mut [u8]) -> Result<(), SymmError> {
		self.0.decrypt_nopad(content)?;
		Ok(())
	}
}


/// Reusable encoder/decoder for Aes256 in Ctr mode and no padding
pub struct AesCtr256(Aes256Ctr);

impl AesCtr256 {

	/// New encoder/decoder
	#[inline]
	pub fn new(key: &[u8], iv: &[u8]) -> Result<Self, SymmError> {
		Ok(AesCtr256(
			Aes256Ctr::new(GenericArray::from_slice(key), GenericArray::from_slice(iv))
		))
	}

	#[inline]
	pub fn encrypt(&mut self, content: &mut[u8]) -> Result<(), SymmError> {
		self.0.try_apply_keystream(content)?;
		Ok(())
	}

	#[inline]
	pub fn decrypt(&mut self, content: &mut[u8]) -> Result<(), SymmError> {
		self.0.try_apply_keystream(content)?;
		Ok(())
	}
}

/// Encrypt a message (CTR mode).
///
/// Key (`k`) length and initialisation vector (`iv`) length have to be 16 bytes each.
/// An error is returned if the input lengths are invalid.
/// If possible please use `inplace_encrypt_128_ctr` to avoid a slice copy.
pub fn encrypt_128_ctr(k: &[u8], iv: &[u8], plain: &[u8], dest: &mut [u8]) -> Result<(), SymmError> {
	let mut encryptor = Aes128Ctr::new(
		GenericArray::from_slice(k),
		GenericArray::from_slice(iv),
	);
	&mut dest[..plain.len()].copy_from_slice(plain);
	encryptor.try_apply_keystream(dest)?;
	Ok(())

}

/// An error is returned if the input lengths are invalid.
pub fn inplace_encrypt_128_ctr(k: &[u8], iv: &[u8], data: &mut [u8]) -> Result<(), SymmError> {
	let mut encryptor = Aes128Ctr::new(
		GenericArray::from_slice(k),
		GenericArray::from_slice(iv),
	);
	encryptor.try_apply_keystream(data)?;
	Ok(())

}

/// Decrypt a message (CTR mode).
///
/// Key (`k`) length and initialisation vector (`iv`) length have to be 16 bytes each.
/// An error is returned if the input lengths are invalid.
/// If possible please use `inplace_decrypt_128_ctr` instead.
pub fn decrypt_128_ctr(k: &[u8], iv: &[u8], encrypted: &[u8], dest: &mut [u8]) -> Result<(), SymmError> {
	let mut encryptor = Aes128Ctr::new(
		GenericArray::from_slice(k),
		GenericArray::from_slice(iv),
	);

	&mut dest[..encrypted.len()].copy_from_slice(encrypted);
	encryptor.try_apply_keystream(dest)?;
	Ok(())
}

/// Decrypt a message (CTR mode).
///
/// Key (`k`) length and initialisation vector (`iv`) length have to be 16 bytes each.
/// An error is returned if the input lengths are invalid.
pub fn inplace_decrypt_128_ctr(k: &[u8], iv: &[u8], data: &mut [u8]) -> Result<(), SymmError> {
	let mut encryptor = Aes128Ctr::new(
		GenericArray::from_slice(k),
		GenericArray::from_slice(iv),
	);

	encryptor.try_apply_keystream(data)?;
	Ok(())
}


#[cfg(test)]
fn encrypt_128_cbc(k: &[u8], iv: &[u8], plain: &[u8], dest: &mut [u8]) -> Result<(), SymmError> {
	let encryptor = Cbc::<Aes128, Pkcs7>::new_varkey(k, GenericArray::from_slice(iv))?;
	&mut dest[..plain.len()].copy_from_slice(plain);
	encryptor.encrypt_pad(dest, plain.len())?;
	Ok(())
}


/// Decrypt a message (CBC mode).
///
/// Key (`k`) length and initialisation vector (`iv`) length have to be 16 bytes each.
/// An error is returned if the input lengths are invalid.
pub fn decrypt_128_cbc(k: &[u8], iv: &[u8], encrypted: &[u8], dest: &mut [u8]) -> Result<usize, SymmError> {
	let encryptor = Cbc::<Aes128, Pkcs7>::new_varkey(k, GenericArray::from_slice(iv))?;
	&mut dest[..encrypted.len()].copy_from_slice(encrypted);
	let unpad_length = {
		encryptor.decrypt_pad(&mut dest[..encrypted.len()])?.len()
	};
	Ok(unpad_length)
}


// retrocomptibility test
#[test]
pub fn test_aes_short() -> Result<(),SymmError> {
	let key = include_bytes!("../test/key1");
	let salt = include_bytes!("../test/salt1");
	let content = include_bytes!("../test/content");
	let ctr_enc = include_bytes!("../test/result_128_ctr");
	let cbc_enc = include_bytes!("../test/result_128_cbc");
	let mut dest = vec![0;110];
	let mut dest_padded = vec![0;112];
	let mut dest_padded2 = vec![0;128]; // TODO RustLib need an extra 16bytes in dest : looks extra buggy but function is not currently use (keep it private for now)
	encrypt_128_cbc(&key[..16], &salt[..16], content, &mut dest_padded2)?;
	assert!(&dest_padded2[..112] == &cbc_enc[..]);
	//	buffer2.write_all(&dest1[..]).unwrap();
	encrypt_128_ctr(&key[..16], &salt[..16], content, &mut dest)?;
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
