// Copyright 2019 Parity Technologies (UK) Ltd.
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
    pub fn from_sender_salt_and_code(sender: &Address, salt: H256, code: &[u8]) -> (Self, H256) {
        let code_hash = keccak(code);
        let mut buffer = [0u8; 1 + 20 + 32 + 32];
        buffer[0] = 0xff;
        &mut buffer[1..(1 + 20)].copy_from_slice(&sender[..]);
        &mut buffer[(1 + 20)..(1 + 20 + 32)].copy_from_slice(&salt[..]);
        &mut buffer[(1 + 20 + 32)..].copy_from_slice(&code_hash[..]);

        (
            ContractAddress(Address::from(keccak(&buffer[..]))),
            code_hash,
        )
    }

    /// Computes the address of a contract from the sender's address and the code hash
    ///
    /// Used by pwasm create ext.
    pub fn from_sender_and_code(sender: &Address, code: &[u8]) -> (Self, H256) {
        let code_hash = keccak(code);
        let mut buffer = [0u8; 20 + 32];
        &mut buffer[..20].copy_from_slice(&sender[..]);
        &mut buffer[20..].copy_from_slice(&code_hash[..]);

        (
            ContractAddress(Address::from(keccak(&buffer[..]))),
            code_hash,
        )
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
        let expected_address =
            Address::from_str("e33c0c7f7df4809055c3eba6c09cfe4baf1bd9e0").unwrap();
        let expected_code_hash =
            H256::from_str("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470")
                .unwrap();

        let (contract_address, code_hash) =
            ContractAddress::from_sender_salt_and_code(&sender, H256::zero(), &[]);

        assert_eq!(Address::from(contract_address), expected_address);
        assert_eq!(code_hash, expected_code_hash);
    }

    #[test]
    fn test_from_sender_and_code_hash() {
        let code = [0u8, 1, 2, 3];
        let sender = Address::from_str("0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d").unwrap();
        let expected_address =
            Address::from_str("064417880f5680b141ed7fcac031aad40df080b0").unwrap();
        let expected_code_hash =
            H256::from_str("d98f2e8134922f73748703c8e7084d42f13d2fa1439936ef5a3abcf5646fe83f")
                .unwrap();

        let (contract_address, code_hash) = ContractAddress::from_sender_and_code(&sender, &code);

        assert_eq!(Address::from(contract_address), expected_address);
        assert_eq!(code_hash, expected_code_hash);
    }
}
