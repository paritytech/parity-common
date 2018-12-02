#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "impl-serde", not(feature = "std")))]
compile_error!("Feature \"impl-serde\" requires feature \"std\" to build.");

#[cfg(all(feature = "impl-rlp", not(feature = "std")))]
compile_error!("Feature \"impl-rlp\" requires feature \"std\" to build.");

#[macro_use]
extern crate uint;

#[macro_use]
extern crate fixed_hash;

#[cfg(feature = "impl-serde")]
#[macro_use]
extern crate impl_serde;

#[cfg(feature = "impl-codec")]
#[macro_use]
extern crate impl_codec;

#[cfg(feature = "impl-rlp")]
#[macro_use]
extern crate impl_rlp;

construct_uint! {
	/// Little-endian 256-bit integer type.
	#[derive(Copy, Clone, Eq, PartialEq, Hash)]
	pub struct U256(4);
}
#[cfg(feature = "impl-serde")] impl_uint_serde!(U256, 4);
#[cfg(feature = "impl-codec")] impl_uint_codec!(U256, 4);
#[cfg(feature = "impl-rlp")] impl_uint_rlp!(U256, 4);

construct_fixed_hash!{
	/// Fixed-size uninterpreted hash type with 20 bytes (160 bits) size.
	pub struct H160(20);
}
#[cfg(feature = "impl-serde")] impl_fixed_hash_serde!(H160, 20);
#[cfg(feature = "impl-codec")] impl_fixed_hash_codec!(H160, 20);
#[cfg(feature = "impl-rlp")] impl_fixed_hash_rlp!(H160, 20);

construct_fixed_hash!{
	/// Fixed-size uninterpreted hash type with 32 bytes (256 bits) size.
	pub struct H256(32);
}
#[cfg(feature = "impl-serde")] impl_fixed_hash_serde!(H256, 32);
#[cfg(feature = "impl-codec")] impl_fixed_hash_codec!(H256, 32);
#[cfg(feature = "impl-rlp")] impl_fixed_hash_rlp!(H256, 32);

construct_fixed_hash!{
	/// Fixed-size uninterpreted hash type with 64 bytes (512 bits) size.
	pub struct H512(64);
}
#[cfg(feature = "impl-serde")] impl_fixed_hash_serde!(H512, 64);
#[cfg(feature = "impl-codec")] impl_fixed_hash_codec!(H512, 64);
#[cfg(feature = "impl-rlp")] impl_fixed_hash_rlp!(H512, 64);
