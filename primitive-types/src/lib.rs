#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "uint-serde", not(feature = "std")))]
compile_error!("Feature \"uint-serde\" requires feature \"std\" to build.");

#[macro_use]
extern crate uint;

#[cfg(feature = "uint-serde")]
#[macro_use]
extern crate uint_serde;

#[cfg(feature = "uint-codec")]
#[macro_use]
extern crate uint_codec;

#[cfg(feature = "uint-rlp")]
#[macro_use]
extern crate uint_rlp;

#[macro_use]
extern crate fixed_hash;

construct_uint!(U256, 4);
#[cfg(feature = "uint-serde")] impl_uint_serde!(U256, 4);
#[cfg(feature = "uint-codec")] impl_uint_codec!(U256, 4);
#[cfg(feature = "uint-rlp")] impl_uint_rlp!(U256, 4);
