use super::*;
#[cfg(not(feature = "std"))]
use alloc::{
	borrow::{Cow, ToOwned},
	string::{String, ToString},
};
#[cfg(feature = "std")]
use std::borrow::Cow;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};

impl JsonSchema for H160 {
	fn schema_name() -> Cow<'static, str> {
		"HexEncoded20Bytes".into()
	}

	fn json_schema(_: &mut SchemaGenerator) -> Schema {
		json_schema!({
			"description": "Hex encoded 20 bytes",
			"pattern": "^0(x|X)[a-fA-F0-9]{40}$",
		})
	}
}

impl JsonSchema for U256 {
	fn schema_name() -> Cow<'static, str> {
		"U256String".into()
	}

	fn json_schema(_: &mut SchemaGenerator) -> Schema {
		json_schema!({
			"description": "256-bit Unsigned Integer",
			"pattern": "^(0|[1-9][0-9]{0,77})$",
		})
	}
}

#[cfg(test)]
#[cfg(any(feature = "serde", feature = "serde_no_std"))]
mod tests {
	use crate::{H160, U256};
	#[cfg(not(feature = "std"))]
	use alloc::string::String;
	use jsonschema::Draft;
	use schemars::JsonSchema;

	#[test]
	fn hex_encoded_20_bytes() {
		let schema = H160::json_schema(&mut schemars::SchemaGenerator::default());
		let schema_json = serde_json::to_value(&schema).unwrap();
		let schema = jsonschema::Validator::options()
			.with_draft(Draft::Draft7)
			.build(&schema_json)
			.unwrap();
		let value = serde_json::to_value("0x55086adeca661185c437d92b9818e6eda6d0d047").unwrap();
		schema
			.validate(&value)
			.map_err(|e| e.into_iter().map(|e| e.to_string()).collect::<Vec<_>>())
			.unwrap();
		let value = serde_json::to_value("0X0E9C8DA9FD4BDD3281879D9E328D8D74D02558CC").unwrap();
		assert!(schema.validate(&value).is_ok());

		let value = serde_json::to_value("42").unwrap();
		assert!(schema.validate(&value).is_err());
	}

	#[test]
	fn u256() {
		let schema = U256::json_schema(&mut schemars::SchemaGenerator::default());
		let schema_json = serde_json::to_value(&schema).unwrap();
		let schema = jsonschema::Validator::options()
			.with_draft(Draft::Draft7)
			.build(&schema_json)
			.unwrap();
		let addr = serde_json::to_value("42").unwrap();
		assert!(schema.validate(&addr).is_ok());
		let addr = serde_json::to_value(['1'; 79].into_iter().collect::<String>()).unwrap();
		assert!(schema.validate(&addr).is_err());
	}
}
