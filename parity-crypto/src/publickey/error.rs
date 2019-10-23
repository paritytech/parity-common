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

//! Module specific errors

use std::{fmt, result, error::Error as StdError};
use crate::error::SymmError;

/// Module specific errors
#[derive(Debug)]
pub enum Error {
	/// secp256k1 enc error
	Secp(secp256k1::Error),
	/// Invalid secret key
	InvalidSecretKey,
	/// Invalid public key
	InvalidPublicKey,
	/// Invalid address
	InvalidAddress,
	/// Invalid EC signature
	InvalidSignature,
	/// Invalid AES message
	InvalidMessage,
	/// IO Error
	Io(std::io::Error),
	/// Symmetric encryption error
	Symm(SymmError),
	/// Custom
	Custom(String),
}

impl StdError for Error {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Error::Secp(secp_err) => Some(secp_err),
			Error::Io(err) => Some(err),
			Error::Symm(symm_err) => Some(symm_err),
			_ => None,
		}
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
		match self {
			Error::Secp(err) => write!(f, "secp error: {}", err),
			Error::InvalidSecretKey => write!(f, "invalid secret key"),
			Error::InvalidPublicKey => write!(f, "invalid public key"),
			Error::InvalidAddress => write!(f, "invalid address"),
			Error::InvalidSignature => write!(f, "invalid EC signature"),
			Error::InvalidMessage => write!(f, "invalid AES message"),
			Error::Io(err) => write!(f, "I/O error: {}", err),
			Error::Symm(err) => write!(f, "symmetric encryption error: {}", err),
			Error::Custom(err) => write!(f, "custom crypto error: {}", err),
		}
	}
}

impl Into<String> for Error {
	fn into(self) -> String {
		format!("{}", self)
	}
}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Error {
		Error::Io(err)
	}
}

impl From<SymmError> for Error {
	fn from(err: SymmError) -> Error {
		Error::Symm(err)
	}
}

impl From<secp256k1::Error> for Error {
	fn from(e: secp256k1::Error) -> Error {
		match e {
			secp256k1::Error::InvalidMessage => Error::InvalidMessage,
			secp256k1::Error::InvalidPublicKey => Error::InvalidPublicKey,
			secp256k1::Error::InvalidSecretKey => Error::InvalidSecretKey,
			_ => Error::InvalidSignature,
		}
	}
}
