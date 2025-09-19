// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Tests for scale-info feature of primitive-types.

use primitive_types::{H256, U256};
use scale_info::{build::Fields, Path, Type, TypeInfo};

#[test]
fn u256_scale_info() {
	assert_eq!(U256::type_info(), scale_info::TypeDefPrimitive::U256.into());
}

#[test]
fn h256_scale_info() {
	let r#type = Type::builder()
		.path(Path::new("H256", "primitive_types"))
		.composite(Fields::unnamed().field(|f| f.ty::<[u8; 32]>().type_name("[u8; 32]")));

	assert_eq!(H256::type_info(), r#type.into());
}
