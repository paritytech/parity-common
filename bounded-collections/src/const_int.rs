// Copyright (C) Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::{Get, TypedGet};
use core::marker::PhantomData;

// Numbers which have constant upper and lower bounds.
trait ConstBounded<T> {
	const MIN: T;
	const MAX: T;
}

macro_rules! impl_const_bounded {
	($bound:ty, $t:ty) => {
		impl ConstBounded<$bound> for $t {
			const MIN: $bound = <$t>::MIN as $bound;
			const MAX: $bound = <$t>::MAX as $bound;
		}
	};
}

impl_const_bounded!(u128, u8);
impl_const_bounded!(u128, u16);
impl_const_bounded!(u128, u32);
impl_const_bounded!(u128, u64);
impl_const_bounded!(u128, u128);
impl_const_bounded!(u128, usize);

impl_const_bounded!(i128, i8);
impl_const_bounded!(i128, i16);
impl_const_bounded!(i128, i32);
impl_const_bounded!(i128, i64);
impl_const_bounded!(i128, i128);

// Check whether a unsigned integer is within the bounds of a type.
struct CheckOverflowU128<T: ConstBounded<u128>, const N: u128>(PhantomData<T>);

impl<T: ConstBounded<u128>, const N: u128> CheckOverflowU128<T, N> {
	const ASSERTION: () = assert!(N >= T::MIN && N <= T::MAX);
}

// Check whether an integer is within the bounds of a type.
struct CheckOverflowI128<T: ConstBounded<i128>, const N: i128>(PhantomData<T>);

impl<T: ConstBounded<i128>, const N: i128> CheckOverflowI128<T, N> {
	const ASSERTION: () = assert!(N >= T::MIN && N <= T::MAX);
}

/// Const getter for unsigned integers.
///
/// # Compile-time checks
///
/// ```compile_fail
/// # use bounded_collections::{ConstUint, Get};
/// let _ = <ConstUint<256> as Get<u8>>::get();
/// ```
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
	($t:ident, $check:ident, $bound:ty, $target:ty) => {
		impl<const N: $bound> Get<$target> for $t<N> {
			fn get() -> $target {
				let _ = <$check<$target, N>>::ASSERTION;
				N as $target
			}
		}
		impl<const N: $bound> Get<Option<$target>> for $t<N> {
			fn get() -> Option<$target> {
				let _ = <$check<$target, N>>::ASSERTION;
				Some(N as $target)
			}
		}
	};
}

impl_const_int!(ConstUint, CheckOverflowU128, u128, u8);
impl_const_int!(ConstUint, CheckOverflowU128, u128, u16);
impl_const_int!(ConstUint, CheckOverflowU128, u128, u32);
impl_const_int!(ConstUint, CheckOverflowU128, u128, u64);
impl_const_int!(ConstUint, CheckOverflowU128, u128, u128);
impl_const_int!(ConstUint, CheckOverflowU128, u128, usize);

impl_const_int!(ConstInt, CheckOverflowI128, i128, i8);
impl_const_int!(ConstInt, CheckOverflowI128, i128, i16);
impl_const_int!(ConstInt, CheckOverflowI128, i128, i32);
impl_const_int!(ConstInt, CheckOverflowI128, i128, i64);
impl_const_int!(ConstInt, CheckOverflowI128, i128, i128);

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
