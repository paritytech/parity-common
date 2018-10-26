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

//! rng adapter for wasm in browser (using websys crate)

use web_sys::Crypto;
use rand::{ CryptoRng, RngCore, Error, ErrorKind };
use std::fmt;
use std::mem::transmute;

#[derive(Clone)]
pub struct OsRng;

impl fmt::Debug for OsRng {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		"OsRng using webcrypto rng source".fmt(f)
	}
}

impl OsRng {
	pub fn new() -> Result<OsRng, Error> {
		Ok(OsRng)
	}
}

impl CryptoRng for OsRng {}

// current buffer usage is quite ineficient
impl RngCore for OsRng {

	fn next_u32(&mut self) -> u32 {
		let result: u32 = 0;
		let mut buf: [u8; 4] = unsafe { transmute(result) };
		let crypto: Crypto = web_sys::window().unwrap().crypto().unwrap();
		crypto.get_random_values_with_u8_array(&mut buf[..]).expect("Not able to operate without random source.");
		unsafe { transmute(buf) }
	}

	fn next_u64(&mut self) -> u64 {
		let result: u64 = 0;
		let mut buf: [u8; 8] = unsafe { transmute(result) };
		let crypto: Crypto = web_sys::window().unwrap().crypto().unwrap();
		crypto.get_random_values_with_u8_array(&mut buf[..]).expect("Not able to operate without random source.");
		unsafe { transmute(buf) }
	}

	fn fill_bytes(&mut self, dest: &mut [u8]) {
		let crypto: Crypto = web_sys::window().unwrap().crypto().unwrap();
		crypto.get_random_values_with_u8_array(dest).expect("Not able to operate without random source.");
	}

	fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
		if let Some(window) = web_sys::window() {
			let crypto = window.crypto()
				.map_err(|_jsval|Error::new(ErrorKind::Unexpected, "Error accessing webcrypto in browser"))?;
			crypto.get_random_values_with_u8_array(dest)
				.map_err(|_jsval|Error::new(ErrorKind::Unexpected, "Error getting random value from webcrypto"))?;
			Ok(())
		} else {
			Err(Error::new(ErrorKind::Unavailable, "error getting window"))
		}
	}

}
 
