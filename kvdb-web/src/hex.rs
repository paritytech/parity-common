// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Serde bytes <-> hex (de)serializer

use serde::{Serialize, Deserialize, Serializer, Deserializer, de};
use std::ops::Deref;

fn as_hex<T, S>(key: &T, serializer: S) -> Result<S::Ok, S::Error>
	where T: AsRef<[u8]>,
		  S: Serializer
{
	serializer.serialize_str(hex::encode(key.as_ref()).as_str())
}

fn from_hex<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
	where D: Deserializer<'de>
{
	String::deserialize(deserializer)
		.and_then(|string|
			hex::decode(&string).map(Into::into)
				.map_err(|err| de::Error::custom(err.to_string()))
		)
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(transparent)]
pub(crate) struct BytesHexEncoding {
	#[serde(serialize_with = "as_hex", deserialize_with = "from_hex")]
	pub inner: Vec<u8>,
}

impl<T: Into<Vec<u8>>> From<T> for BytesHexEncoding {
	fn from(t: T) -> Self {
		BytesHexEncoding {
			inner: t.into(),
		}
	}
}

impl std::borrow::Borrow<[u8]> for BytesHexEncoding {
	fn borrow(&self) -> &[u8] {
		self.inner.as_slice()
	}
}

impl Deref for BytesHexEncoding {
	type Target = Vec<u8>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}
