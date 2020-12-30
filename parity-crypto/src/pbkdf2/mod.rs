// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub struct Salt<'a>(pub &'a [u8]);
pub struct Secret<'a>(pub &'a [u8]);

pub fn sha256(iter: u32, salt: Salt<'_>, sec: Secret<'_>, out: &mut [u8; 32]) {
	pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(sec.0, salt.0, iter, out)
}

pub fn sha512(iter: u32, salt: Salt<'_>, sec: Secret<'_>, out: &mut [u8; 64]) {
	pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha512>>(sec.0, salt.0, iter, out)
}

#[cfg(test)]
mod test;
