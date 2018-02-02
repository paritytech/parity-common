extern crate serde;

use std::fmt;
use serde::{de, Serializer, Deserializer};

static CHARS: &'static[u8] = b"0123456789abcdef";

fn to_hex(bytes: &[u8], skip_leading_zero: bool) -> String {
    let mut v = Vec::with_capacity(2 + bytes.len() * 2);
    v.push('0' as u8);
    v.push('x' as u8);

    let first_nibble = bytes[0] >> 4;
    if first_nibble != 0 || !skip_leading_zero {
        v.push(CHARS[first_nibble as usize]);
    }
    v.push(CHARS[(bytes[0] & 0xf) as usize]);

    for &byte in bytes.iter().skip(1) {
        v.push(CHARS[(byte >> 4) as usize]);
        v.push(CHARS[(byte & 0xf) as usize]);
    }

    unsafe {
        String::from_utf8_unchecked(v)
    }
}

/// Serializes a slice of bytes.
pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> where
	S: Serializer,
{
	serializer.serialize_str(&to_hex(bytes, false))
}

/// Serialize a slice of bytes as uint.
///
/// The representation will have all leading zeros trimmed.
pub fn serialize_uint<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> where
	S: Serializer,
{
	let non_zero = bytes.iter().take_while(|b| **b == 0).count();
	let bytes = &bytes[non_zero..];
	if bytes.is_empty() {
		return serializer.serialize_str("0x0");
	}

    let string = to_hex(bytes, true);
	serializer.serialize_str(&*string)
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
			ExpectedLen::Exact(ref v) => write!(fmt, "length of {}", v.len() * 2),
			ExpectedLen::Between(min, ref v) => write!(fmt, "length between ({}; {}]", min * 2, v.len() * 2),
		}
	}
}

/// Deserialize into vector of bytes with additional size check.
/// Returns number of bytes written.
pub fn deserialize_check_len<'a, 'de, D>(deserializer: D, len: ExpectedLen<'a>) -> Result<usize, D::Error> where
	D: Deserializer<'de>,
{
	struct Visitor<'a> {
		len: ExpectedLen<'a>,
	}

	impl<'a, 'b> de::Visitor<'b> for Visitor<'a> {
		type Value = usize;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			write!(formatter, "a 0x-prefixed hex string with {}", self.len)
		}

		fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
			if v.len() < 2  || &v[0..2] != "0x" {
				return Err(E::custom("prefix is missing"))
			}

			let is_len_valid = match self.len {
				ExpectedLen::Exact(ref slice) => v.len() == 2 * slice.len() + 2,
				ExpectedLen::Between(min, ref slice) => v.len() <= 2 * slice.len() + 2 && v.len() > 2 * min + 2,
			};

			if !is_len_valid {
				return Err(E::invalid_length(v.len() - 2, &self))
			}

			let bytes = match self.len {
				ExpectedLen::Exact(slice) => slice,
                ExpectedLen::Between(_, slice) => slice,
            };

            let mut modulus = v.len() % 2;
            let mut buf = 0;
            let mut pos = 0;
            for (idx, byte) in v.bytes().enumerate().skip(2) {
                buf <<= 4;

                match byte {
                    b'A'...b'F' => buf |= byte - b'A' + 10,
                    b'a'...b'f' => buf |= byte - b'a' + 10,
                    b'0'...b'9' => buf |= byte - b'0',
                    b' '|b'\r'|b'\n'|b'\t' => {
                        buf >>= 4;
                        continue
                    }
                    _ => {
                        let ch = v[idx..].chars().next().unwrap();
                        return Err(E::custom(&format!("invalid hex character: {}, at {}", ch, idx)))
                    }
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

		fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
			self.visit_str(&v)
		}
	}

	deserializer.deserialize_str(Visitor { len })
}
