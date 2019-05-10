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

use rscrypt;
use block_modes;
use aes_ctr;
use std::error::Error as StdError;

quick_error! {
	#[derive(Debug)]
	pub enum Error {
		Scrypt(e: ScryptError) {
			cause(e)
			from()
		}
		Symm(e: SymmError) {
			cause(e)
			from()
		}
		AsymShort(det: &'static str) {
			description(det)
		}
		AsymFull(e: Box<dyn StdError + Send>) {
			cause(&**e)
			description(e.description())
		}
	}
}

impl Into<std::io::Error> for Error {
	fn into(self) -> std::io::Error {
		std::io::Error::new(std::io::ErrorKind::Other, format!("Crypto error: {}",self))
	}
}

quick_error! {
	#[derive(Debug)]
	pub enum ScryptError {
		// log(N) < r / 16
		InvalidN {
			display("Invalid N argument of the scrypt encryption")
		}
		// p <= (2^31-1 * 32)/(128 * r)
		InvalidP {
			display("Invalid p argument of the scrypt encryption")
		}
		ScryptParam(e: rscrypt::errors::InvalidParams) {
			display("invalid params for scrypt: {}", e)
			cause(e)
			from()
		}
		ScryptLength(e: rscrypt::errors::InvalidOutputLen) {
			display("invalid scrypt output length: {}", e)
			cause(e)
			from()
		}
	}
}


quick_error! {
	#[derive(Debug)]
	pub enum SymmError wraps PrivSymmErr {
		Offset(x: usize) {
			display("offset {} greater than slice length", x)
		}
		BlockMode(e: block_modes::BlockModeError) {
			display("symmetric crypto error")
			from()
		}
		KeyStream(e: aes_ctr::stream_cipher::LoopError) {
			display("ctr key stream ended")
			from()
		}
		InvalidKeyLength(e: block_modes::InvalidKeyIvLength) {
			display("Error with RustCrypto key length : {}", e)
			from()
		}
		SymmetricCrypto {
			display("symmetric crypto error")
			from()
		}
	}
}

impl SymmError {
	#[cfg(feature = "aead")]
	pub(crate) fn offset_error(x: usize) -> SymmError {
		SymmError(PrivSymmErr::Offset(x))
	}

	#[cfg(feature = "aead")]
	pub(crate) fn symmetric_crypto_error() -> SymmError {
		SymmError(PrivSymmErr::SymmetricCrypto)
	}
}

impl From<block_modes::BlockModeError> for SymmError {
	fn from(e: block_modes::BlockModeError) -> SymmError {
		SymmError(PrivSymmErr::BlockMode(e))
	}
}

impl From<block_modes::InvalidKeyIvLength> for SymmError {
	fn from(e: block_modes::InvalidKeyIvLength) -> SymmError {
		SymmError(PrivSymmErr::InvalidKeyLength(e))
	}
}

impl From<aes_ctr::stream_cipher::LoopError> for SymmError {
	fn from(e: aes_ctr::stream_cipher::LoopError) -> SymmError {
		SymmError(PrivSymmErr::KeyStream(e))
	}
}
