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

pub struct Salt<'a>(pub &'a [u8]);
pub struct Secret<'a>(pub &'a [u8]);

pub fn sha256(iter: u32, salt: Salt, sec: Secret, out: &mut [u8; 32]) {
	pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(sec.0, salt.0, iter as usize, out)
}

pub fn sha512(iter: u32, salt: Salt, sec: Secret, out: &mut [u8; 64]) {
	pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha512>>(sec.0, salt.0, iter as usize, out)
}

#[cfg(test)]
mod test;
