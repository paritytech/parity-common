// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! num-traits support for uint.

#![no_std]

#[doc(hidden)]
pub use num_traits;

#[doc(hidden)]
pub use integer_sqrt;

#[doc(hidden)]
pub use uint;

/// Add num-traits support to an integer created by `construct_uint!`.
#[macro_export]
macro_rules! impl_uint_num_traits {
	($name: ident, $len: expr) => {
		impl $crate::num_traits::bounds::Bounded for $name {
			#[inline]
			fn min_value() -> Self {
				Self::zero()
			}

			#[inline]
			fn max_value() -> Self {
				Self::max_value()
			}
		}

		impl $crate::num_traits::sign::Unsigned for $name {}

		impl $crate::num_traits::identities::ConstZero for $name {
			const ZERO: Self = Self::zero();
		}

		impl $crate::num_traits::identities::ConstOne for $name {
			const ONE: Self = Self::one();
		}

		impl $crate::num_traits::identities::Zero for $name {
			#[inline]
			fn zero() -> Self {
				Self::zero()
			}

			#[inline]
			fn is_zero(&self) -> bool {
				self.is_zero()
			}
		}

		impl $crate::num_traits::identities::One for $name {
			#[inline]
			fn one() -> Self {
				Self::one()
			}
		}

		impl $crate::num_traits::Num for $name {
			type FromStrRadixErr = $crate::uint::FromStrRadixErr;

			fn from_str_radix(txt: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
				Self::from_str_radix(txt, radix)
			}
		}

		impl $crate::integer_sqrt::IntegerSquareRoot for $name {
			fn integer_sqrt_checked(&self) -> Option<Self> {
				Some(self.integer_sqrt())
			}
		}

		impl $crate::num_traits::ops::bytes::FromBytes for $name {
			type Bytes = [u8; $len * 8];

			fn from_be_bytes(bytes: &Self::Bytes) -> Self {
				Self::from_big_endian(&bytes[..])
			}

			fn from_le_bytes(bytes: &Self::Bytes) -> Self {
				Self::from_little_endian(&bytes[..])
			}
		}

		impl $crate::num_traits::ops::bytes::ToBytes for $name {
			type Bytes = [u8; $len * 8];

			fn to_be_bytes(&self) -> Self::Bytes {
				self.to_big_endian()
			}

			fn to_le_bytes(&self) -> Self::Bytes {
				self.to_little_endian()
			}
		}

		impl $crate::num_traits::ops::checked::CheckedAdd for $name {
			#[inline]
			fn checked_add(&self, v: &Self) -> Option<Self> {
				$name::checked_add(*self, *v)
			}
		}

		impl $crate::num_traits::ops::checked::CheckedSub for $name {
			#[inline]
			fn checked_sub(&self, v: &Self) -> Option<Self> {
				$name::checked_sub(*self, *v)
			}
		}

		impl $crate::num_traits::ops::checked::CheckedDiv for $name {
			#[inline]
			fn checked_div(&self, v: &Self) -> Option<Self> {
				$name::checked_div(*self, *v)
			}
		}

		impl $crate::num_traits::ops::checked::CheckedMul for $name {
			#[inline]
			fn checked_mul(&self, v: &Self) -> Option<Self> {
				$name::checked_mul(*self, *v)
			}
		}

		impl $crate::num_traits::ops::checked::CheckedNeg for $name {
			#[inline]
			fn checked_neg(&self) -> Option<Self> {
				Self::checked_neg(*self)
			}
		}

		impl $crate::num_traits::ops::checked::CheckedRem for $name {
			#[inline]
			fn checked_rem(&self, v: &Self) -> Option<Self> {
				Self::checked_rem(*self, *v)
			}
		}

		impl $crate::num_traits::ops::saturating::Saturating for $name {
			#[inline]
			fn saturating_add(self, v: Self) -> Self {
				Self::saturating_add(self, v)
			}

			#[inline]
			fn saturating_sub(self, v: Self) -> Self {
				Self::saturating_sub(self, v)
			}
		}

		impl $crate::num_traits::ops::saturating::SaturatingAdd for $name {
			#[inline]
			fn saturating_add(&self, v: &Self) -> Self {
				Self::saturating_add(*self, *v)
			}
		}

		impl $crate::num_traits::ops::saturating::SaturatingMul for $name {
			#[inline]
			fn saturating_mul(&self, v: &Self) -> Self {
				Self::saturating_mul(*self, *v)
			}
		}

		impl $crate::num_traits::ops::saturating::SaturatingSub for $name {
			#[inline]
			fn saturating_sub(&self, v: &Self) -> Self {
				Self::saturating_sub(*self, *v)
			}
		}

		impl $crate::num_traits::pow::Pow<Self> for $name {
			type Output = Self;

			fn pow(self, rhs: Self) -> Self {
				Self::pow(self, rhs)
			}
		}
	};
}
