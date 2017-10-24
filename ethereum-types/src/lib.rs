#![cfg_attr(not(feature="std"), no_std)]

#[cfg(feature="std")]
extern crate core;
#[macro_use]
extern crate crunchy;
#[macro_use]
extern crate uint as uint_crate;

mod uint;

pub use uint::{U128, U256, U512};
