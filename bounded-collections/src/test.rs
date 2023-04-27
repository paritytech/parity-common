// Copyright 2023 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Tests for the `bounded-collections` crate.

#![cfg(test)]

use crate::*;
use core::fmt::Debug;

#[test]
#[allow(path_statements)]
fn const_impl_default_clone_debug() {
	struct ImplsDefault<T: Default + Clone + Debug>(T);

	ImplsDefault::<ConstBool<true>>;
	ImplsDefault::<ConstBool<false>>;
	ImplsDefault::<ConstU8<255>>;
	ImplsDefault::<ConstU16<50>>;
	ImplsDefault::<ConstU32<10>>;
	ImplsDefault::<ConstU64<99>>;
	ImplsDefault::<ConstU128<100>>;
	ImplsDefault::<ConstI8<-127>>;
	ImplsDefault::<ConstI16<-50>>;
	ImplsDefault::<ConstI32<-10>>;
	ImplsDefault::<ConstI64<-99>>;
	ImplsDefault::<ConstI128<-100>>;
}

#[test]
#[cfg(feature = "std")]
fn const_debug_fmt() {
	assert_eq!(format!("{:?}", ConstBool::<true> {}), "ConstBool<true>");
	assert_eq!(format!("{:?}", ConstBool::<false> {}), "ConstBool<false>");
	assert_eq!(format!("{:?}", ConstU8::<255> {}), "ConstU8<255>");
	assert_eq!(format!("{:?}", ConstU16::<50> {}), "ConstU16<50>");
	assert_eq!(format!("{:?}", ConstU32::<10> {}), "ConstU32<10>");
	assert_eq!(format!("{:?}", ConstU64::<99> {}), "ConstU64<99>");
	assert_eq!(format!("{:?}", ConstU128::<100> {}), "ConstU128<100>");
	assert_eq!(format!("{:?}", ConstI8::<-127> {}), "ConstI8<-127>");
	assert_eq!(format!("{:?}", ConstI16::<-50> {}), "ConstI16<-50>");
	assert_eq!(format!("{:?}", ConstI32::<-10> {}), "ConstI32<-10>");
	assert_eq!(format!("{:?}", ConstI64::<-99> {}), "ConstI64<-99>");
	assert_eq!(format!("{:?}", ConstI128::<-100> {}), "ConstI128<-100>");
}
