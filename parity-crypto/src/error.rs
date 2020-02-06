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

use std::{error::Error as StdError, fmt, result};

#[derive(Debug)]
pub enum Error {
	Scrypt(ScryptError),
	Symm(SymmError),
}

#[derive(Debug)]
pub enum ScryptError {
	// log(N) < r / 16
	InvalidN,
	// p <= (2^31-1 * 32)/(128 * r)
	InvalidP,
	ScryptParam(scrypt::errors::InvalidParams),
	ScryptLength(scrypt::errors::InvalidOutputLen),
}

#[derive(Debug)]
pub struct SymmError(PrivSymmErr);

#[derive(Debug)]
enum PrivSymmErr {
	BlockMode(block_modes::BlockModeError),
	KeyStream(aes_ctr::stream_cipher::LoopError),
	InvalidKeyLength(block_modes::InvalidKeyIvLength),
}

impl StdError for Error {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Error::Scrypt(scrypt_err) => Some(scrypt_err),
			Error::Symm(symm_err) => Some(symm_err),
		}
	}
}

impl StdError for ScryptError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			ScryptError::ScryptParam(err) => Some(err),
			ScryptError::ScryptLength(err) => Some(err),
			_ => None,
		}
	}
}

impl StdError for SymmError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match &self.0 {
			PrivSymmErr::BlockMode(err) => Some(err),
			PrivSymmErr::InvalidKeyLength(err) => Some(err),
			_ => None,
		}
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> result::Result<(), fmt::Error> {
		match self {
			Error::Scrypt(err) => write!(f, "scrypt error: {}", err),
			Error::Symm(err) => write!(f, "symm error: {}", err),
		}
	}
}

impl fmt::Display for ScryptError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> result::Result<(), fmt::Error> {
		match self {
			ScryptError::InvalidN => write!(f, "invalid n argument"),
			ScryptError::InvalidP => write!(f, "invalid p argument"),
			ScryptError::ScryptParam(err) => write!(f, "invalid params: {}", err),
			ScryptError::ScryptLength(err) => write!(f, "invalid output length: {}", err),
		}
	}
}

impl fmt::Display for SymmError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> result::Result<(), fmt::Error> {
		match self {
			SymmError(PrivSymmErr::BlockMode(err)) => write!(f, "block cipher error: {}", err),
			SymmError(PrivSymmErr::KeyStream(err)) => write!(f, "ctr key stream ended: {}", err),
			SymmError(PrivSymmErr::InvalidKeyLength(err)) => write!(f, "block cipher key length: {}", err),
		}
	}
}

impl Into<std::io::Error> for Error {
	fn into(self) -> std::io::Error {
		std::io::Error::new(std::io::ErrorKind::Other, format!("Crypto error: {}", self))
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

impl From<scrypt::errors::InvalidParams> for ScryptError {
	fn from(e: scrypt::errors::InvalidParams) -> ScryptError {
		ScryptError::ScryptParam(e)
	}
}

impl From<scrypt::errors::InvalidOutputLen> for ScryptError {
	fn from(e: scrypt::errors::InvalidOutputLen) -> ScryptError {
		ScryptError::ScryptLength(e)
	}
}

impl From<ScryptError> for Error {
	fn from(e: ScryptError) -> Error {
		Error::Scrypt(e)
	}
}

impl From<SymmError> for Error {
	fn from(e: SymmError) -> Error {
		Error::Symm(e)
	}
}
