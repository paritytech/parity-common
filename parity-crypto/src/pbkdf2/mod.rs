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

use std::num::NonZeroU32;

use ring;

pub struct Salt<'a>(pub &'a [u8]);
pub struct Secret<'a>(pub &'a [u8]);

pub fn sha256(iter: NonZeroU32, salt: Salt, sec: Secret, out: &mut [u8; 32]) {
	ring::pbkdf2::derive(&ring::digest::SHA256, iter, salt.0, sec.0, &mut out[..])
}

pub fn sha512(iter: NonZeroU32, salt: Salt, sec: Secret, out: &mut [u8; 64]) {
	ring::pbkdf2::derive(&ring::digest::SHA512, iter, salt.0, sec.0, &mut out[..])
}


#[cfg(test)]
mod test;
