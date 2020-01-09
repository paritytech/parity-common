// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Tests for type-metadata feature of primitive-types.

use primitive_types::{H256, U256};
use type_metadata::{HasTypeDef, HasTypeId, Namespace, TypeDefTupleStruct, TypeIdCustom, UnnamedField};

#[test]
fn u256_type_metadata() {
	let type_id = TypeIdCustom::new("U256", Namespace::new(vec!["primitive_types"]).unwrap(), vec![]);
	assert_eq!(U256::type_id(), type_id.into());

	let type_def = TypeDefTupleStruct::new(vec![UnnamedField::of::<[u64; 4]>()]).into();
	assert_eq!(U256::type_def(), type_def);
}

#[test]
fn h256_type_metadata() {
	let type_id = TypeIdCustom::new("H256", Namespace::new(vec!["primitive_types"]).unwrap(), vec![]);
	assert_eq!(H256::type_id(), type_id.into());

	let type_def = TypeDefTupleStruct::new(vec![UnnamedField::of::<[u8; 32]>()]).into();
	assert_eq!(H256::type_def(), type_def);
}
