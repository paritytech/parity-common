#![cfg_attr(not(feature="std"), no_std)]

#![cfg_attr(asm_available, feature(asm))]

#[cfg(feature="std")]
extern crate core;
#[macro_use]
extern crate crunchy;
#[macro_use]
extern crate uint as uint_crate;
#[macro_use]
extern crate fixed_hash;
extern crate ethbloom;

mod hash;
mod uint;

pub use uint::{U128, U256, U512};
pub use hash::{H32, H64, H128, H160, H256, H264, H512, H520, H1024};
pub use ethbloom::{Bloom, BloomRef, Input as BloomInput};
pub use fixed_hash::clean_0x;

pub type Address = H160;
pub type Secret = H256;
pub type Public = H512;
pub type Signature = H520;
