# Contract address

Provides a function to create an ethereum contract address.

## Examples

Create an ethereum address from sender and nonce.

```rust
use contract_address::{
	Address, U256, ContractAddress
};
use std::str::FromStr;

let sender = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
let contract_address = ContractAddress::from_sender_and_nonce(&sender, &U256::zero());
```
