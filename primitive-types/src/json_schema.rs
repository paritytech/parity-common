use super::*;
#[cfg(not(feature = "std"))]
use alloc::{
	borrow::ToOwned,
	string::{String, ToString},
};

use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};

impl JsonSchema for H160 {
	fn schema_name() -> String {
		"HexEncoded20Bytes".to_owned()
	}

	fn json_schema(gen: &mut SchemaGenerator) -> Schema {
		let mut schema = gen.subschema_for::<String>().into_object();
		schema.metadata().description = Some("Hex encoded 20 bytes".to_string());
		schema.string().pattern = Some("^0(x|X)[a-fA-F0-9]{40}$".to_string());
		schema.into()
	}
}

impl JsonSchema for U256 {
	fn schema_name() -> String {
		"U256String".to_string()
	}

	fn json_schema(gen: &mut SchemaGenerator) -> Schema {
		let mut schema = gen.subschema_for::<String>().into_object();
		schema.metadata().description = Some("256-bit Unsigned Integer".to_string());
		schema.string().pattern = Some("^(0|[1-9][0-9]{0,77})$".to_string());
		schema.into()
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
		let schema = H160::json_schema(&mut schemars::gen::SchemaGenerator::default());
		let schema_json = serde_json::to_value(&schema).unwrap();
		let schema = jsonschema::JSONSchema::options()
			.with_draft(Draft::Draft7)
			.compile(&schema_json)
			.unwrap();
		let value = serde_json::to_value("0x55086adeca661185c437d92b9818e6eda6d0d047").unwrap();
		assert!(schema.validate(&value).is_ok());
		let value = serde_json::to_value("0X0E9C8DA9FD4BDD3281879D9E328D8D74D02558CC").unwrap();
		assert!(schema.validate(&value).is_ok());

		let value = serde_json::to_value("42").unwrap();
		assert!(schema.validate(&value).is_err());
	}

	#[test]
	fn u256() {
		let schema = U256::json_schema(&mut schemars::gen::SchemaGenerator::default());
		let schema_json = serde_json::to_value(&schema).unwrap();
		let schema = jsonschema::JSONSchema::options()
			.with_draft(Draft::Draft7)
			.compile(&schema_json)
			.unwrap();
		let addr = serde_json::to_value("42").unwrap();
		assert!(schema.validate(&addr).is_ok());
		let addr = serde_json::to_value(['1'; 79].into_iter().collect::<String>()).unwrap();
		assert!(schema.validate(&addr).is_err());
	}
}
