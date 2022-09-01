// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use alloc::{string::String, vec::Vec};
use core::{fmt, result::Result};
use serde::{de, Deserializer, Serializer};

static CHARS: &[u8] = b"0123456789abcdef";

/// Serialize given bytes to a 0x-prefixed hex string.
///
/// If `skip_leading_zero` initial 0s will not be printed out,
/// unless the byte string is empty, in which case `0x0` will be returned.
/// The results are consistent with `serialize_uint` output if the flag is
/// on and `serialize_raw` if the flag is off.
pub fn to_hex(bytes: &[u8], skip_leading_zero: bool) -> String {
	let bytes = if skip_leading_zero {
		let non_zero = bytes.iter().take_while(|b| **b == 0).count();
		let bytes = &bytes[non_zero..];
		if bytes.is_empty() {
			return "0x0".into()
		} else {
			bytes
		}
	} else if bytes.is_empty() {
		return "0x".into()
	} else {
		bytes
	};

	let mut slice = vec![0u8; (bytes.len() + 1) * 2];
	to_hex_raw(&mut slice, bytes, skip_leading_zero).into()
}

fn to_hex_raw<'a>(v: &'a mut [u8], bytes: &[u8], skip_leading_zero: bool) -> &'a str {
	assert!(v.len() > 1 + bytes.len() * 2);

	v[0] = b'0';
	v[1] = b'x';

	let mut idx = 2;
	let first_nibble = bytes[0] >> 4;
	if first_nibble != 0 || !skip_leading_zero {
		v[idx] = CHARS[first_nibble as usize];
		idx += 1;
	}
	v[idx] = CHARS[(bytes[0] & 0xf) as usize];
	idx += 1;

	for &byte in bytes.iter().skip(1) {
		v[idx] = CHARS[(byte >> 4) as usize];
		v[idx + 1] = CHARS[(byte & 0xf) as usize];
		idx += 2;
	}

	// SAFETY: all characters come either from CHARS or "0x", therefore valid UTF8
	unsafe { core::str::from_utf8_unchecked(&v[0..idx]) }
}

/// Decoding bytes from hex string error.
#[derive(Debug, PartialEq, Eq)]
pub enum FromHexError {
	/// The `0x` prefix is missing.
	#[deprecated(since = "0.3.2", note = "We support non 0x-prefixed hex strings")]
	MissingPrefix,
	/// Invalid (non-hex) character encountered.
	InvalidHex {
		/// The unexpected character.
		character: char,
		/// Index of that occurrence.
		index: usize,
	},
}

#[cfg(feature = "std")]
impl std::error::Error for FromHexError {}

impl fmt::Display for FromHexError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			#[allow(deprecated)]
			Self::MissingPrefix => write!(fmt, "0x prefix is missing"),
			Self::InvalidHex { character, index } => write!(fmt, "invalid hex character: {}, at {}", character, index),
		}
	}
}

/// Decode given (both 0x-prefixed or not) hex string into a vector of bytes.
///
/// Returns an error if non-hex characters are present.
pub fn from_hex(v: &str) -> Result<Vec<u8>, FromHexError> {
	let (v, stripped) = v.strip_prefix("0x").map_or((v, false), |v| (v, true));

	let mut bytes = vec![0u8; (v.len() + 1) / 2];
	from_hex_raw(v, &mut bytes, stripped)?;
	Ok(bytes)
}

/// Decode given 0x-prefix-stripped hex string into provided slice.
/// Used internally by `from_hex` and `deserialize_check_len`.
///
/// The method will panic if `bytes` have incorrect length (make sure to allocate enough beforehand).
fn from_hex_raw(v: &str, bytes: &mut [u8], stripped: bool) -> Result<usize, FromHexError> {
	let bytes_len = v.len();
	let mut modulus = bytes_len % 2;
	let mut buf = 0;
	let mut pos = 0;
	for (index, byte) in v.bytes().enumerate() {
		buf <<= 4;

		match byte {
			b'A'..=b'F' => buf |= byte - b'A' + 10,
			b'a'..=b'f' => buf |= byte - b'a' + 10,
			b'0'..=b'9' => buf |= byte - b'0',
			b' ' | b'\r' | b'\n' | b'\t' => {
				buf >>= 4;
				continue
			},
			b => {
				let character = char::from(b);
				return Err(FromHexError::InvalidHex { character, index: index + if stripped { 2 } else { 0 } })
			},
		}

		modulus += 1;
		if modulus == 2 {
			modulus = 0;
			bytes[pos] = buf;
			pos += 1;
		}
	}

	Ok(pos)
}

/// Serializes a slice of bytes.
pub fn serialize_raw<S>(slice: &mut [u8], bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	if bytes.is_empty() {
		serializer.serialize_str("0x")
	} else {
		serializer.serialize_str(to_hex_raw(slice, bytes, false))
	}
}

/// Serializes a slice of bytes.
pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let mut slice = vec![0u8; (bytes.len() + 1) * 2];
	serialize_raw(&mut slice, bytes, serializer)
}

/// Serialize a slice of bytes as uint.
///
/// The representation will have all leading zeros trimmed.
pub fn serialize_uint<S>(slice: &mut [u8], bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let non_zero = bytes.iter().take_while(|b| **b == 0).count();
	let bytes = &bytes[non_zero..];
	if bytes.is_empty() {
		serializer.serialize_str("0x0")
	} else {
		serializer.serialize_str(to_hex_raw(slice, bytes, true))
	}
}

/// Expected length of bytes vector.
#[derive(Debug, PartialEq, Eq)]
pub enum ExpectedLen<'a> {
	/// Exact length in bytes.
	Exact(&'a mut [u8]),
	/// A bytes length between (min; slice.len()].
	Between(usize, &'a mut [u8]),
}

impl<'a> fmt::Display for ExpectedLen<'a> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ExpectedLen::Exact(ref v) => write!(fmt, "{} bytes", v.len()),
			ExpectedLen::Between(min, ref v) => write!(fmt, "between ({}; {}] bytes", min, v.len()),
		}
	}
}

/// Deserialize into vector of bytes.  This will allocate an O(n) intermediate
/// string.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
	D: Deserializer<'de>,
{
	struct Visitor;

	impl<'b> de::Visitor<'b> for Visitor {
		type Value = Vec<u8>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			write!(formatter, "a (both 0x-prefixed or not) hex string or byte array")
		}

		fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
			from_hex(v).map_err(E::custom)
		}

		fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
			self.visit_str(&v)
		}

		fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
			Ok(v.to_vec())
		}

		fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
			Ok(v)
		}

		fn visit_seq<A: de::SeqAccess<'b>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
			let mut bytes = vec![];
			while let Some(n) = seq.next_element::<u8>()? {
				bytes.push(n);
			}
			Ok(bytes)
		}

		fn visit_newtype_struct<D: Deserializer<'b>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
			deserializer.deserialize_bytes(self)
		}
	}

	deserializer.deserialize_str(Visitor)
}

/// Deserialize into vector of bytes with additional size check.
/// Returns number of bytes written.
pub fn deserialize_check_len<'a, 'de, D>(deserializer: D, len: ExpectedLen<'a>) -> Result<usize, D::Error>
where
	D: Deserializer<'de>,
{
	struct Visitor<'a> {
		len: ExpectedLen<'a>,
	}

	impl<'a, 'b> de::Visitor<'b> for Visitor<'a> {
		type Value = usize;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			write!(formatter, "a (both 0x-prefixed or not) hex string or byte array containing {}", self.len)
		}

		fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
			let (v, stripped) = v.strip_prefix("0x").map_or((v, false), |v| (v, true));

			let len = v.len();
			let is_len_valid = match self.len {
				ExpectedLen::Exact(ref slice) => len == 2 * slice.len(),
				ExpectedLen::Between(min, ref slice) => len <= 2 * slice.len() && len > 2 * min,
			};

			if !is_len_valid {
				return Err(E::invalid_length(v.len(), &self))
			}

			let bytes = match self.len {
				ExpectedLen::Exact(slice) => slice,
				ExpectedLen::Between(_, slice) => slice,
			};

			from_hex_raw(v, bytes, stripped).map_err(E::custom)
		}

		fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
			self.visit_str(&v)
		}

		fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
			let len = v.len();
			let is_len_valid = match self.len {
				ExpectedLen::Exact(ref slice) => len == slice.len(),
				ExpectedLen::Between(min, ref slice) => len <= slice.len() && len > min,
			};

			if !is_len_valid {
				return Err(E::invalid_length(v.len(), &self))
			}

			let bytes = match self.len {
				ExpectedLen::Exact(slice) => slice,
				ExpectedLen::Between(_, slice) => slice,
			};

			bytes[..len].copy_from_slice(v);
			Ok(len)
		}

		fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
			self.visit_bytes(&v)
		}

		fn visit_seq<A: de::SeqAccess<'b>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
			let mut v = vec![];
			while let Some(n) = seq.next_element::<u8>()? {
				v.push(n);
			}
			self.visit_byte_buf(v)
		}

		fn visit_newtype_struct<D: Deserializer<'b>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
			deserializer.deserialize_bytes(self)
		}
	}

	deserializer.deserialize_str(Visitor { len })
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_derive::{Deserialize, Serialize};

	#[derive(Serialize, Deserialize)]
	struct Bytes(#[serde(with = "super")] Vec<u8>);

	#[test]
	fn should_not_fail_on_short_string_with_prefix() {
		let a: Bytes = serde_json::from_str("\"0x\"").unwrap();
		let b: Bytes = serde_json::from_str("\"0x1\"").unwrap();
		let c: Bytes = serde_json::from_str("\"0x12\"").unwrap();
		let d: Bytes = serde_json::from_str("\"0x123\"").unwrap();
		let e: Bytes = serde_json::from_str("\"0x1234\"").unwrap();
		let f: Bytes = serde_json::from_str("\"0x12345\"").unwrap();

		assert!(a.0.is_empty());
		assert_eq!(b.0, vec![1]);
		assert_eq!(c.0, vec![0x12]);
		assert_eq!(d.0, vec![0x1, 0x23]);
		assert_eq!(e.0, vec![0x12, 0x34]);
		assert_eq!(f.0, vec![0x1, 0x23, 0x45]);
	}

	#[test]
	fn should_not_fail_on_other_strings_with_prefix() {
		let a: Bytes =
			serde_json::from_str("\"0x7f864e18e3dd8b58386310d2fe0919eef27c6e558564b7f67f22d99d20f587\"").unwrap();
		let b: Bytes =
			serde_json::from_str("\"0x7f864e18e3dd8b58386310d2fe0919eef27c6e558564b7f67f22d99d20f587b\"").unwrap();
		let c: Bytes =
			serde_json::from_str("\"0x7f864e18e3dd8b58386310d2fe0919eef27c6e558564b7f67f22d99d20f587b4\"").unwrap();

		assert_eq!(a.0.len(), 31);
		assert_eq!(b.0.len(), 32);
		assert_eq!(c.0.len(), 32);
	}

	#[test]
	fn should_not_fail_on_short_string_without_prefix() {
		let a: Bytes = serde_json::from_str("\"\"").unwrap();
		let b: Bytes = serde_json::from_str("\"1\"").unwrap();
		let c: Bytes = serde_json::from_str("\"12\"").unwrap();
		let d: Bytes = serde_json::from_str("\"123\"").unwrap();
		let e: Bytes = serde_json::from_str("\"1234\"").unwrap();
		let f: Bytes = serde_json::from_str("\"12345\"").unwrap();

		assert!(a.0.is_empty());
		assert_eq!(b.0, vec![1]);
		assert_eq!(c.0, vec![0x12]);
		assert_eq!(d.0, vec![0x1, 0x23]);
		assert_eq!(e.0, vec![0x12, 0x34]);
		assert_eq!(f.0, vec![0x1, 0x23, 0x45]);
	}

	#[test]
	fn should_not_fail_on_other_strings_without_prefix() {
		let a: Bytes =
			serde_json::from_str("\"7f864e18e3dd8b58386310d2fe0919eef27c6e558564b7f67f22d99d20f587\"").unwrap();
		let b: Bytes =
			serde_json::from_str("\"7f864e18e3dd8b58386310d2fe0919eef27c6e558564b7f67f22d99d20f587b\"").unwrap();
		let c: Bytes =
			serde_json::from_str("\"7f864e18e3dd8b58386310d2fe0919eef27c6e558564b7f67f22d99d20f587b4\"").unwrap();

		assert_eq!(a.0.len(), 31);
		assert_eq!(b.0.len(), 32);
		assert_eq!(c.0.len(), 32);
	}

	#[test]
	fn should_serialize_and_deserialize_empty_bytes() {
		let bytes = Bytes(Vec::new());

		let data = serde_json::to_string(&bytes).unwrap();

		assert_eq!("\"0x\"", &data);

		let deserialized: Bytes = serde_json::from_str(&data).unwrap();
		assert!(deserialized.0.is_empty())
	}

	#[test]
	fn should_encode_to_and_from_hex_with_prefix() {
		assert_eq!(to_hex(&[0, 1, 2], true), "0x102");
		assert_eq!(to_hex(&[0, 1, 2], false), "0x000102");
		assert_eq!(to_hex(&[0], true), "0x0");
		assert_eq!(to_hex(&[], true), "0x0");
		assert_eq!(to_hex(&[], false), "0x");
		assert_eq!(to_hex(&[0], false), "0x00");
		assert_eq!(from_hex("0x0102"), Ok(vec![1, 2]));
		assert_eq!(from_hex("0x102"), Ok(vec![1, 2]));
		assert_eq!(from_hex("0xf"), Ok(vec![0xf]));
	}

	#[test]
	fn should_decode_hex_without_prefix() {
		assert_eq!(from_hex("0102"), Ok(vec![1, 2]));
		assert_eq!(from_hex("102"), Ok(vec![1, 2]));
		assert_eq!(from_hex("f"), Ok(vec![0xf]));
	}

	#[test]
	fn should_deserialize_from_owned_bytes() {
		type BytesDeserializer<'a> = serde::de::value::BytesDeserializer<'a, serde::de::value::Error>;

		// using `deserialize` to decode owned bytes.
		let des = BytesDeserializer::new(&[1, 2, 3, 4, 5]);
		let deserialized: Vec<u8> = deserialize(des).unwrap();
		assert_eq!(deserialized, vec![1, 2, 3, 4, 5]);

		// using `deserialize` to decode owned bytes into buffer with fixed length.
		let des = BytesDeserializer::new(&[1, 2, 3, 4, 5]);
		let mut output = vec![0, 0, 0, 0, 0];
		let expected_len = ExpectedLen::Exact(&mut *output);
		let n = deserialize_check_len(des, expected_len).unwrap();
		assert_eq!(n, 5);
		assert_eq!(output, vec![1, 2, 3, 4, 5]);

		// using `deserialize` to decode owned bytes into buffer with min/max length.
		let des = BytesDeserializer::new(&[1, 2, 3]);
		let mut output = vec![0, 0, 0, 0, 0];
		let expected_len = ExpectedLen::Between(2, &mut *output);
		let n = deserialize_check_len(des, expected_len).unwrap();
		assert_eq!(n, 3);
		assert_eq!(output, vec![1, 2, 3, 0, 0]);
	}

	#[test]
	fn should_deserialize_from_borrowed_bytes() {
		type BytesDeserializer<'a> = serde::de::value::BorrowedBytesDeserializer<'a, serde::de::value::Error>;

		// using `deserialize` to decode borrowed bytes.
		let des = BytesDeserializer::new(&[1, 2, 3, 4, 5]);
		let deserialized: Vec<u8> = deserialize(des).unwrap();
		assert_eq!(deserialized, vec![1, 2, 3, 4, 5]);

		// using `deserialize` to decode borrowed bytes into buffer with fixed length.
		let des = BytesDeserializer::new(&[1, 2, 3, 4, 5]);
		let mut output = vec![0, 0, 0, 0, 0];
		let expected_len = ExpectedLen::Exact(&mut *output);
		let n = deserialize_check_len(des, expected_len).unwrap();
		assert_eq!(n, 5);
		assert_eq!(output, vec![1, 2, 3, 4, 5]);

		// using `deserialize` to decode borrowed bytes into buffer with min/max length.
		let des = BytesDeserializer::new(&[1, 2, 3]);
		let mut output = vec![0, 0, 0, 0, 0];
		let expected_len = ExpectedLen::Between(2, &mut *output);
		let n = deserialize_check_len(des, expected_len).unwrap();
		assert_eq!(n, 3);
		assert_eq!(output, vec![1, 2, 3, 0, 0]);
	}

	#[test]
	fn should_deserialize_from_u8_sequence() {
		use serde::de::value::SeqDeserializer;

		// using `deserialize` to decode a sequence of bytes.
		let des = SeqDeserializer::<_, serde::de::value::Error>::new([1u8, 2, 3, 4, 5].into_iter());
		let deserialized: Vec<u8> = deserialize(des).unwrap();
		assert_eq!(deserialized, vec![1, 2, 3, 4, 5]);

		// using `deserialize` to decode a sequence of bytes into a buffer with fixed length.
		let des = SeqDeserializer::<_, serde::de::value::Error>::new([1u8, 2, 3, 4, 5].into_iter());
		let mut output = vec![0, 0, 0, 0, 0];
		let expected_len = ExpectedLen::Exact(&mut *output);
		let n = deserialize_check_len(des, expected_len).unwrap();
		assert_eq!(n, 5);
		assert_eq!(output, vec![1, 2, 3, 4, 5]);

		// using `deserialize` to decode a sequence of bytes into a buffer with min/max length.
		let des = SeqDeserializer::<_, serde::de::value::Error>::new([1u8, 2, 3].into_iter());
		let mut output = vec![0, 0, 0, 0, 0];
		let expected_len = ExpectedLen::Between(2, &mut *output);
		let n = deserialize_check_len(des, expected_len).unwrap();
		assert_eq!(n, 3);
		assert_eq!(output, vec![1, 2, 3, 0, 0]);
	}
}
