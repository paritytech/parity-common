// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Errors that can occur when working with IndexedDB.

use std::fmt;

/// An error that occurred when working with IndexedDB.
#[derive(Clone, PartialEq, Debug)]
pub enum Error {
	/// Accessing a Window has failed.
	/// Are we in a WebWorker?
	WindowNotAvailable,
	/// IndexedDB is not supported by your browser.
	NotSupported(String),
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
			Error::__Nonexhaustive => unreachable!(),
		}
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match *self {
			Error::WindowNotAvailable => write!(f, "Accessing a Window has failed"),
			Error::NotSupported(ref err) => write!(f, "IndexedDB is not supported by your browser: {}", err,),
			Error::__Nonexhaustive => unreachable!(),
		}
	}
}
