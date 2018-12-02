#[doc(hidden)]
pub extern crate serde;

#[doc(hidden)]
pub extern crate rustc_hex;

#[doc(hidden)]
pub mod serialize;

#[macro_export]
macro_rules! impl_uint_serde {
	($name: ident, $len: expr) => {
		impl $crate::serde::Serialize for $name {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: $crate::serde::Serializer {
				let mut bytes = [0u8; $len * 8];
				self.to_big_endian(&mut bytes);
				$crate::serialize::serialize_uint(&bytes, serializer)
			}
		}

		impl<'de> $crate::serde::Deserialize<'de> for $name {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: $crate::serde::Deserializer<'de> {
				$crate::serialize::deserialize_check_len(deserializer, $crate::serialize::ExpectedLen::Between(0, $len * 8))
					.map(|x| (&*x).into())
			}
		}
	}
}

#[macro_export]
macro_rules! impl_fixed_hash_serde {
	($name: ident, $len: expr) => {
		impl $crate::serde::Serialize for $name {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: $crate::serde::Serializer {
				$crate::serialize::serialize(&self.0, serializer)
			}
		}

		impl<'de> $crate::serde::Deserialize<'de> for $name {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: $crate::serde::Deserializer<'de> {
				$crate::serialize::deserialize_check_len(deserializer, $crate::serialize::ExpectedLen::Exact($len))
					.map(|x| $name::from_slice(&x))
			}
		}
	}
}
