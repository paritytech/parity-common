// Copyright 2023 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Collection types that have an upper limit on how many elements that they can contain, and
//! supporting traits that aid in defining the limit.

#![cfg_attr(not(feature = "std"), no_std)]

pub extern crate alloc;

pub mod bounded_btree_map;
pub mod bounded_btree_set;
pub mod bounded_vec;
pub mod weak_bounded_vec;

mod test;

pub use bounded_btree_map::BoundedBTreeMap;
pub use bounded_btree_set::BoundedBTreeSet;
pub use bounded_vec::{BoundedSlice, BoundedVec};
pub use weak_bounded_vec::WeakBoundedVec;

/// A trait for querying a single value from a type defined in the trait.
///
/// It is not required that the value is constant.
pub trait TypedGet {
	/// The type which is returned.
	type Type;
	/// Return the current value.
	fn get() -> Self::Type;
}

/// A trait for querying a single value from a type.
///
/// It is not required that the value is constant.
pub trait Get<T> {
	/// Return the current value.
	fn get() -> T;
}

impl<T: Default> Get<T> for () {
	fn get() -> T {
		T::default()
	}
}

/// Implement Get by returning Default for any type that implements Default.
pub struct GetDefault;
impl<T: Default> Get<T> for GetDefault {
	fn get() -> T {
		T::default()
	}
}

macro_rules! impl_const_get {
	($name:ident, $t:ty) => {
		/// Const getter for a basic type.
		#[derive(Default, Clone)]
		pub struct $name<const T: $t>;

		#[cfg(feature = "std")]
		impl<const T: $t> core::fmt::Debug for $name<T> {
			fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
				fmt.write_str(&format!("{}<{}>", stringify!($name), T))
			}
		}
		#[cfg(not(feature = "std"))]
		impl<const T: $t> core::fmt::Debug for $name<T> {
			fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
				fmt.write_str("<wasm:stripped>")
			}
		}
		impl<const T: $t> Get<$t> for $name<T> {
			fn get() -> $t {
				T
			}
		}
		impl<const T: $t> Get<Option<$t>> for $name<T> {
			fn get() -> Option<$t> {
				Some(T)
			}
		}
		impl<const T: $t> TypedGet for $name<T> {
			type Type = $t;
			fn get() -> $t {
				T
			}
		}
	};
}

impl_const_get!(ConstBool, bool);
impl_const_get!(ConstU8, u8);
impl_const_get!(ConstU16, u16);
impl_const_get!(ConstU32, u32);
impl_const_get!(ConstU64, u64);
impl_const_get!(ConstU128, u128);
impl_const_get!(ConstI8, i8);
impl_const_get!(ConstI16, i16);
impl_const_get!(ConstI32, i32);
impl_const_get!(ConstI64, i64);
impl_const_get!(ConstI128, i128);

/// Try and collect into a collection `C`.
pub trait TryCollect<C> {
	/// The error type that gets returned when a collection can't be made from `self`.
	type Error;
	/// Consume self and try to collect the results into `C`.
	///
	/// This is useful in preventing the undesirable `.collect().try_into()` call chain on
	/// collections that need to be converted into a bounded type (e.g. `BoundedVec`).
	fn try_collect(self) -> Result<C, Self::Error>;
}

/// Create new implementations of the [`Get`](crate::Get) trait.
///
/// The so-called parameter type can be created in four different ways:
///
/// - Using `const` to create a parameter type that provides a `const` getter. It is required that
///   the `value` is const.
///
/// - Declare the parameter type without `const` to have more freedom when creating the value.
///
/// NOTE: A more substantial version of this macro is available in `frame_support` crate which
/// allows mutable and persistant variants.
///
/// # Examples
///
/// ```
/// # use bounded_collections::Get;
/// # use bounded_collections::parameter_types;
/// // This function cannot be used in a const context.
/// fn non_const_expression() -> u64 { 99 }
///
/// const FIXED_VALUE: u64 = 10;
/// parameter_types! {
///    pub const Argument: u64 = 42 + FIXED_VALUE;
///    /// Visibility of the type is optional
///    OtherArgument: u64 = non_const_expression();
/// }
///
/// trait Config {
///    type Parameter: Get<u64>;
///    type OtherParameter: Get<u64>;
/// }
///
/// struct Runtime;
/// impl Config for Runtime {
///    type Parameter = Argument;
///    type OtherParameter = OtherArgument;
/// }
/// ```
///
/// # Invalid example:
///
/// ```compile_fail
/// # use bounded_collections::Get;
/// # use bounded_collections::parameter_types;
/// // This function cannot be used in a const context.
/// fn non_const_expression() -> u64 { 99 }
///
/// parameter_types! {
///    pub const Argument: u64 = non_const_expression();
/// }
/// ```
#[macro_export]
macro_rules! parameter_types {
	(
		$( #[ $attr:meta ] )*
		$vis:vis const $name:ident: $type:ty = $value:expr;
		$( $rest:tt )*
	) => (
		$( #[ $attr ] )*
		$vis struct $name;
		$crate::parameter_types!(@IMPL_CONST $name , $type , $value);
		$crate::parameter_types!( $( $rest )* );
	);
	(
		$( #[ $attr:meta ] )*
		$vis:vis $name:ident: $type:ty = $value:expr;
		$( $rest:tt )*
	) => (
		$( #[ $attr ] )*
		$vis struct $name;
		$crate::parameter_types!(@IMPL $name, $type, $value);
		$crate::parameter_types!( $( $rest )* );
	);
	() => ();
	(@IMPL_CONST $name:ident, $type:ty, $value:expr) => {
		impl $name {
			/// Returns the value of this parameter type.
			pub const fn get() -> $type {
				$value
			}
		}

		impl<I: From<$type>> $crate::Get<I> for $name {
			fn get() -> I {
				I::from(Self::get())
			}
		}

		impl $crate::TypedGet for $name {
			type Type = $type;
			fn get() -> $type {
				Self::get()
			}
		}
	};
	(@IMPL $name:ident, $type:ty, $value:expr) => {
		impl $name {
			/// Returns the value of this parameter type.
			pub fn get() -> $type {
				$value
			}
		}

		impl<I: From<$type>> $crate::Get<I> for $name {
			fn get() -> I {
				I::from(Self::get())
			}
		}

		impl $crate::TypedGet for $name {
			type Type = $type;
			fn get() -> $type {
				Self::get()
			}
		}
	};
}

/// Build a bounded vec from the given literals.
///
/// The type of the outcome must be known.
///
/// Will not handle any errors and just panic if the given literals cannot fit in the corresponding
/// bounded vec type. Thus, this is only suitable for testing and non-consensus code.
#[macro_export]
#[cfg(feature = "std")]
macro_rules! bounded_vec {
	($ ($values:expr),* $(,)?) => {
		{
			$crate::alloc::vec![$($values),*].try_into().unwrap()
		}
	};
	( $value:expr ; $repetition:expr ) => {
		{
			$crate::alloc::vec![$value ; $repetition].try_into().unwrap()
		}
	}
}

/// Build a bounded btree-map from the given literals.
///
/// The type of the outcome must be known.
///
/// Will not handle any errors and just panic if the given literals cannot fit in the corresponding
/// bounded vec type. Thus, this is only suitable for testing and non-consensus code.
#[macro_export]
#[cfg(feature = "std")]
macro_rules! bounded_btree_map {
	($ ( $key:expr => $value:expr ),* $(,)?) => {
		{
			$crate::TryCollect::<$crate::BoundedBTreeMap<_, _, _>>::try_collect(
				$crate::alloc::vec![$(($key, $value)),*].into_iter()
			).unwrap()
		}
	};
}
