// Copyright (C) Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::{Get, TypedGet};

trait ConstBounded<T> {
	const MIN: T;
	const MAX: T;
}

macro_rules! impl_const_bounded {
	($t:ty, $type:ty) => {
		impl ConstBounded<$type> for $t {
			const MIN: $type = <$t>::MIN as $type;
			const MAX: $type = <$t>::MAX as $type;
		}
	};
}

impl_const_bounded!(u8, u128);
impl_const_bounded!(u16, u128);
impl_const_bounded!(u32, u128);
impl_const_bounded!(u64, u128);
impl_const_bounded!(u128, u128);
impl_const_bounded!(usize, u128);

impl_const_bounded!(i8, i128);
impl_const_bounded!(i16, i128);
impl_const_bounded!(i32, i128);
impl_const_bounded!(i64, i128);
impl_const_bounded!(i128, i128);

struct CheckOverflowU128<const N: u128, T: ConstBounded<u128>>(T);

impl<const N: u128, T: ConstBounded<u128>> CheckOverflowU128<N, T> {
	const ASSERTION: () = assert!(N <= T::MAX && N >= T::MIN);
}

struct CheckOverflowI128<const N: i128, T: ConstBounded<i128>>(T);

impl<const N: i128, T: ConstBounded<i128>> CheckOverflowI128<N, T> {
	const ASSERTION: () = assert!(N <= T::MAX && N >= T::MIN);
}

/// Const getter for unsigned integers.
#[derive(Default, Clone)]
pub struct ConstUint<const N: u128>;

#[cfg(feature = "std")]
impl<const N: u128> std::fmt::Debug for ConstUint<N> {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		fmt.write_str(&format!("{}<{}>", stringify!(ConstUint), N))
	}
}
#[cfg(not(feature = "std"))]
impl<const N: u128> core::fmt::Debug for ConstUint<N> {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
		fmt.write_str("<wasm:stripped>")
	}
}

impl<const N: u128> TypedGet for ConstUint<N> {
	type Type = u128;
	fn get() -> u128 {
		N
	}
}

/// Const getter for signed integers.
#[derive(Default, Clone)]
pub struct ConstInt<const N: i128>;

#[cfg(feature = "std")]
impl<const N: i128> std::fmt::Debug for ConstInt<N> {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		fmt.write_str(&format!("{}<{}>", stringify!(ConstInt), N))
	}
}
#[cfg(not(feature = "std"))]
impl<const N: i128> core::fmt::Debug for ConstInt<N> {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
		fmt.write_str("<wasm:stripped>")
	}
}

impl<const N: i128> TypedGet for ConstInt<N> {
	type Type = i128;
	fn get() -> i128 {
		N
	}
}

macro_rules! impl_const_int {
	($type:ident, $value:ty, $overflow:ident, $t:ty) => {
		impl<const N: $value> Get<$t> for $type<N> {
			fn get() -> $t {
				let _ = $overflow::<N, $t>::ASSERTION;
				N as $t
			}
		}
		impl<const N: $value> Get<Option<$t>> for $type<N> {
			fn get() -> Option<$t> {
				let _ = $overflow::<N, $t>::ASSERTION;
				Some(N as $t)
			}
		}
	};
}

impl_const_int!(ConstUint, u128, CheckOverflowU128, u8);
impl_const_int!(ConstUint, u128, CheckOverflowU128, u16);
impl_const_int!(ConstUint, u128, CheckOverflowU128, u32);
impl_const_int!(ConstUint, u128, CheckOverflowU128, u64);
impl_const_int!(ConstUint, u128, CheckOverflowU128, u128);
impl_const_int!(ConstUint, u128, CheckOverflowU128, usize);

impl_const_int!(ConstInt, i128, CheckOverflowI128, i8);
impl_const_int!(ConstInt, i128, CheckOverflowI128, i16);
impl_const_int!(ConstInt, i128, CheckOverflowI128, i32);
impl_const_int!(ConstInt, i128, CheckOverflowI128, i64);
impl_const_int!(ConstInt, i128, CheckOverflowI128, i128);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn const_uint_works() {
		assert_eq!(<ConstUint<42> as Get<u8>>::get(), 42);
		assert_eq!(<ConstUint<42> as Get<Option<u8>>>::get(), Some(42));
		assert_eq!(<ConstUint<42> as Get<u16>>::get(), 42);
		assert_eq!(<ConstUint<42> as Get<u32>>::get(), 42);
		assert_eq!(<ConstUint<42> as Get<u64>>::get(), 42);
		assert_eq!(<ConstUint<42> as Get<u128>>::get(), 42);
		assert_eq!(<ConstUint<42> as Get<usize>>::get(), 42);
		assert_eq!(<ConstUint<42> as TypedGet>::get(), 42);
		// compile-time error
		// assert_eq!(<ConstUint<256> as Get<u8>>::get() as u128, 256);
	}

	#[test]
	fn const_int_works() {
		assert_eq!(<ConstInt<-42> as Get<i8>>::get(), -42);
		assert_eq!(<ConstInt<-42> as Get<Option<i8>>>::get(), Some(-42));
		assert_eq!(<ConstInt<-42> as Get<i16>>::get(), -42);
		assert_eq!(<ConstInt<-42> as Get<i32>>::get(), -42);
		assert_eq!(<ConstInt<-42> as Get<i64>>::get(), -42);
		assert_eq!(<ConstInt<-42> as Get<i128>>::get(), -42);
		assert_eq!(<ConstInt<-42> as TypedGet>::get(), -42);
	}
}
