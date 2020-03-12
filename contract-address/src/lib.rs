// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(feature = "external_doc", feature(external_doc))]
#![cfg_attr(feature = "external_doc", doc(include = "../README.md"))]

pub use ethereum_types::{Address, H256, U256};
use keccak_hash::keccak;
use rlp::RlpStream;
use std::ops::Deref;

/// Represents an ethereum contract address
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct ContractAddress(Address);

impl ContractAddress {
	/// Computes the address of a contract from the sender's address and the transaction nonce
	pub fn from_sender_and_nonce(sender: &Address, nonce: &U256) -> Self {
		let mut stream = RlpStream::new_list(2);
		stream.append(sender);
		stream.append(nonce);

		ContractAddress(Address::from(keccak(stream.as_raw())))
	}

	/// Computes the address of a contract from the sender's address, the salt and code hash
	///
	/// pWASM `create2` scheme and EIP-1014 CREATE2 scheme
	pub fn from_sender_salt_and_code(sender: &Address, salt: H256, code_hash: H256) -> Self {
		let mut buffer = [0u8; 1 + 20 + 32 + 32];
		buffer[0] = 0xff;
		&mut buffer[1..(1 + 20)].copy_from_slice(&sender[..]);
		&mut buffer[(1 + 20)..(1 + 20 + 32)].copy_from_slice(&salt[..]);
		&mut buffer[(1 + 20 + 32)..].copy_from_slice(&code_hash[..]);

		ContractAddress(Address::from(keccak(&buffer[..])))
	}

	/// Computes the address of a contract from the sender's address and the code hash
	///
	/// Used by pwasm create ext.
	pub fn from_sender_and_code(sender: &Address, code_hash: H256) -> Self {
		let mut buffer = [0u8; 20 + 32];
		&mut buffer[..20].copy_from_slice(&sender[..]);
		&mut buffer[20..].copy_from_slice(&code_hash[..]);

		ContractAddress(Address::from(keccak(&buffer[..])))
	}
}

impl Deref for ContractAddress {
	type Target = Address;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl From<ContractAddress> for Address {
	fn from(contract_address: ContractAddress) -> Self {
		contract_address.0
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::str::FromStr;

	#[test]
	fn test_from_sender_and_nonce() {
		let sender = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let expected = Address::from_str("3f09c73a5ed19289fb9bdc72f1742566df146f56").unwrap();

		let actual = ContractAddress::from_sender_and_nonce(&sender, &U256::from(88));

		assert_eq!(Address::from(actual), expected);
	}

	#[test]
	fn test_from_sender_salt_and_code_hash() {
		let sender = Address::zero();
		let code_hash = H256::from_str("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470").unwrap();
		let expected_address = Address::from_str("e33c0c7f7df4809055c3eba6c09cfe4baf1bd9e0").unwrap();

		let contract_address = ContractAddress::from_sender_salt_and_code(&sender, H256::zero(), code_hash);

		assert_eq!(Address::from(contract_address), expected_address);
	}

	#[test]
	fn test_from_sender_and_code_hash() {
		let sender = Address::from_str("0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d").unwrap();
		let code_hash = H256::from_str("d98f2e8134922f73748703c8e7084d42f13d2fa1439936ef5a3abcf5646fe83f").unwrap();
		let expected_address = Address::from_str("064417880f5680b141ed7fcac031aad40df080b0").unwrap();

		let contract_address = ContractAddress::from_sender_and_code(&sender, code_hash);

		assert_eq!(Address::from(contract_address), expected_address);
	}
}
