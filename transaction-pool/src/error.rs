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

/// TODO: Consider making Hash generic for Error - legacy of error-chain
/// So the hashes are converted to debug strings for easy display.
type Hash = String;

use std::{error, fmt, result};

/// Transaction Pool Error
#[derive(Debug)]
pub enum Error {
	/// Transaction is already imported
	AlreadyImported(Hash),
	/// Transaction is too cheap to enter the queue
	TooCheapToEnter (Hash, String),
	/// Transaction is too cheap to replace existing transaction that occupies the same slot.
	TooCheapToReplace (Hash, Hash),
}

/// Transaction Pool Result
pub type Result<T> = result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Error::AlreadyImported(h) =>
				write!(f, "[{}] already imported", h),
			Error::TooCheapToEnter(hash, min_score) =>
				write!(f, "[{}] too cheap to enter the pool. Min score: {}", hash, min_score),
			Error::TooCheapToReplace(old_hash, hash) =>
				write!(f, "[{}] too cheap to replace: {}", hash, old_hash),
		}
    }
}

impl error::Error for Error {}

#[cfg(test)]
impl PartialEq for Error {
	fn eq(&self, other: &Self) -> bool {
		use self::Error::*;

		match (self, other) {
			(&AlreadyImported(ref h1), &AlreadyImported(ref h2)) => h1 == h2,
			(&TooCheapToEnter(ref h1, ref s1 ), &TooCheapToEnter (ref h2, ref s2)) => h1 == h2 && s1 == s2,
			(&TooCheapToReplace (ref old1, ref new1), &TooCheapToReplace (ref old2, ref new2)) => old1 == old2 && new1 == new2,
			_ => false,
		}
	}
}
