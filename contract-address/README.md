# Contract address

Provides a function to create an ethereum contract address.

## Examples

Create an ethereum address from sender and nonce.

```rust
use contract_address::{
	Address, U256, contract_address, CreateContractAddress,
};
use std::str::FromStr;

let scheme = CreateContractAddress::FromSenderAndNonce;
let sender = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
let (address, _code_hash) = contract_address(scheme, &sender, &U256::zero(), &[]);
```
