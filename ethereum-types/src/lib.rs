#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate core;
#[macro_use]
extern crate uint as uint_crate;
#[macro_use]
extern crate fixed_hash;
extern crate ethbloom;
extern crate primitive_types;

#[cfg(feature = "serialize")]
#[macro_use]
extern crate impl_serde;

#[macro_use]
extern crate impl_rlp;

#[cfg(test)]
extern crate serde_json;

mod hash;
mod uint;

pub use uint::{U64, U128, U256, U512};
pub use hash::{BigEndianHash, H32, H64, H128, H160, H256, H264, H512, H520};
pub use ethbloom::{Bloom, BloomRef, Input as BloomInput};

pub type Address = H160;
pub type Secret = H256;
pub type Public = H512;
pub type Signature = H520;
