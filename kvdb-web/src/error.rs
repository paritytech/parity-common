// Copyright 2019 Parity Technologies (UK) Ltd.
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

//! Errors that can occur when working with IndexedDB.

use std::fmt;
use wasm_bindgen::JsValue;


/// An error that occurred when working with IndexedDB.
#[derive(Clone, PartialEq, Debug)]
pub enum Error {
	/// Accessing a Window has failed.
	/// Are we in a WebWorker?
	WindowNotAvailable,
    /// IndexedDB is not supported by your browser.
    NotSupported(JsValue),
    /// Commiting a transaction to IndexedDB has failed.
    TransactionFailed(JsValue),
    /// This enum may grow additional variants,
	/// so this makes sure clients don't count on exhaustive matching.
	/// (Otherwise, adding a new variant could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
			Error::WindowNotAvailable => "Accessing a Window has failed",
            Error::NotSupported(_) => "IndexedDB is not supported by your browser",
            Error::TransactionFailed(_) => "Commiting a transaction to IndexedDB has failed",
            Error::__Nonexhaustive => unreachable!(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
			Error::WindowNotAvailable => write!(f, "Accessing a Window has failed"),
            Error::NotSupported(ref err) => write!(
				f,
				"IndexedDB is not supported by your browser: {:?}",
				err,
			),
            Error::TransactionFailed(ref err) => write!(
				f,
				"Commiting a transaction to IndexedDB has failed: {:?}",
				err,
            ),
            Error::__Nonexhaustive => unreachable!(),
        }
    }
}
