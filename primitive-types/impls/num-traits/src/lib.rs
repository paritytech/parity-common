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
		impl $crate::num_traits::sign::Unsigned for $name {}

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
	};
}
