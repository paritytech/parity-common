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

//! secp256k1 for parity.
//! TODO sized u8 array in proto should be usable if we add methods such as U256 -> &[u8;32] to ethereum_types

extern crate secp256k1;

// reexports
pub use self::secp256k1::{
  Error,
  Secp256k1,
  ecdh,
};

pub use self::secp256k1::key::{SecretKey, PublicKey, MINUS_ONE_KEY, ONE_KEY};
pub use self::secp256k1::constants::{SECRET_KEY_SIZE, GENERATOR_X, GENERATOR_Y, CURVE_ORDER};

use self::secp256k1::{
  Message,
  RecoverableSignature,
  RecoveryId,
};

lazy_static! {
	pub static ref SECP256K1: self::secp256k1::Secp256k1 = self::secp256k1::Secp256k1::new();
}


pub fn sign(secret: &[u8], message: &[u8]) -> Result<[u8;65], Error> {
	let context = &SECP256K1;
	let sec = SecretKey::from_slice(context, &secret[..])?;
	let s = context.sign_recoverable(&Message::from_slice(message)?, &sec)?;
	let (rec_id, data) = s.serialize_compact(context);
	let mut data_arr = [0; 65];

	// no need to check if s is low, it always is
	data_arr[0..64].copy_from_slice(&data[0..64]);
	data_arr[64] = rec_id.to_i32() as u8;
	Ok(data_arr)
}

pub fn verify_public(public: &[u8], signature: &[u8], message: &[u8]) -> Result<bool, Error> {
	let context = &SECP256K1;
	let rsig = RecoverableSignature::from_compact(context, &signature[0..64], RecoveryId::from_i32(signature[64] as i32)?)?;
	let sig = rsig.to_standard(context);

	let pdata: [u8; 65] = {
		let mut temp = [4u8; 65];
		temp[1..65].copy_from_slice(&*public);
		temp
	};

	let publ = PublicKey::from_slice(context, &pdata)?;
	match context.verify(&Message::from_slice(message)?, &sig, &publ) {
		Ok(_) => Ok(true),
		Err(Error::IncorrectSignature) => Ok(false),
		Err(x) => Err(Error::from(x))
	}
}

pub fn recover(signature: &[u8], message: &[u8]) -> Result<[u8;64], Error> {
	let context = &SECP256K1;
	let rsig = RecoverableSignature::from_compact(context, &signature[0..64], RecoveryId::from_i32(signature[64] as i32)?)?;
	let pubkey = context.recover(&Message::from_slice(message)?, &rsig)?;
	let serialized = pubkey.serialize_vec(context, false);

  let mut res = [0;64];
  res.copy_from_slice(&serialized[1..65]);
	Ok(res)
}


