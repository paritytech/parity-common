// This file is part of Substrate.

// Copyright (C) 2023 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Shared codec utilities for bounded collections.

/// Struct which allows prepending the compact after reading from an input.
///
/// This is used internally by bounded collections to reconstruct the original
/// input stream after reading the length prefix during decoding.
pub struct PrependCompactInput<'a, I> {
	pub encoded_len: &'a [u8],
	pub read: usize,
	pub inner: &'a mut I,
}

/// Macro to implement Input trait for PrependCompactInput for different codec crates
macro_rules! impl_prepend_compact_input {
	($codec:ident) => {
		use $codec::{Error, Input};

		impl<'a, I: Input> Input for PrependCompactInput<'a, I> {
			fn remaining_len(&mut self) -> Result<Option<usize>, Error> {
				let remaining_compact = self.encoded_len.len().saturating_sub(self.read);
				Ok(self.inner.remaining_len()?.map(|len| len.saturating_add(remaining_compact)))
			}

			fn read(&mut self, into: &mut [u8]) -> Result<(), Error> {
				if into.is_empty() {
					return Ok(());
				}

				let remaining_compact = self.encoded_len.len().saturating_sub(self.read);
				if remaining_compact > 0 {
					let to_read = into.len().min(remaining_compact);
					into[..to_read].copy_from_slice(&self.encoded_len[self.read..][..to_read]);
					self.read += to_read;

					if to_read < into.len() {
						// Buffer not full, keep reading the inner.
						self.inner.read(&mut into[to_read..])
					} else {
						// Buffer was filled by the compact.
						Ok(())
					}
				} else {
					// Prepended compact has been read, just read from inner.
					self.inner.read(into)
				}
			}
		}
	};
}

// Generate implementations for each codec
#[cfg(feature = "scale-codec")]
pub mod scale_codec_impl {
	use super::PrependCompactInput;
	impl_prepend_compact_input!(scale_codec);
}

#[cfg(feature = "jam-codec")]
pub mod jam_codec_impl {
	use super::PrependCompactInput;
	impl_prepend_compact_input!(jam_codec);
}

#[cfg(test)]
mod tests {
	use super::PrependCompactInput;

	/// Macro to generate tests for different codec implementations
	macro_rules! codec_tests {
		($codec:ident, $mod_name:ident) => {
			mod $mod_name {
				use super::PrependCompactInput;
				use $codec::{Compact, Encode, Input};

				#[test]
				fn prepend_compact_input_works() {
					let encoded_len = Compact(3u32).encode();
					let inner = [2, 3, 4];
					let mut input =
						PrependCompactInput { encoded_len: encoded_len.as_ref(), read: 0, inner: &mut &inner[..] };
					assert_eq!(input.remaining_len(), Ok(Some(4)));

					// Passing an empty buffer should leave input unchanged.
					let mut empty_buf = [];
					assert_eq!(input.read(&mut empty_buf), Ok(()));
					assert_eq!(input.remaining_len(), Ok(Some(4)));
					assert_eq!(input.read, 0);

					// Passing a correctly-sized buffer will read correctly.
					let mut buf = [0; 4];
					assert_eq!(input.read(&mut buf), Ok(()));
					assert_eq!(buf[0], encoded_len[0]);
					assert_eq!(buf[1..], inner[..]);
					// And the bookkeeping agrees.
					assert_eq!(input.remaining_len(), Ok(Some(0)));
					assert_eq!(input.read, encoded_len.len());

					// And we can't read more.
					assert!(input.read(&mut buf).is_err());
				}

				#[test]
				fn prepend_compact_input_incremental_read_works() {
					let encoded_len = Compact(3u32).encode();
					let inner = [2, 3, 4];
					let mut input =
						PrependCompactInput { encoded_len: encoded_len.as_ref(), read: 0, inner: &mut &inner[..] };
					assert_eq!(input.remaining_len(), Ok(Some(4)));

					// Compact is first byte - ensure that it fills the buffer when it's more than one.
					let mut buf = [0u8; 2];
					assert_eq!(input.read(&mut buf), Ok(()));
					assert_eq!(buf[0], encoded_len[0]);
					assert_eq!(buf[1], inner[0]);
					assert_eq!(input.remaining_len(), Ok(Some(2)));
					assert_eq!(input.read, encoded_len.len());

					// Check the last two bytes are read correctly.
					assert_eq!(input.read(&mut buf), Ok(()));
					assert_eq!(buf[..], inner[1..]);
					assert_eq!(input.remaining_len(), Ok(Some(0)));
					assert_eq!(input.read, encoded_len.len());

					// And we can't read more.
					assert!(input.read(&mut buf).is_err());
				}
			}
		};
	}

	// Generate tests for each available codec
    #[cfg(feature = "scale-codec")]
	codec_tests!(scale_codec, scale_codec_impl);
    #[cfg(feature = "jam-codec")]
	codec_tests!(jam_codec, jam_codec_impl);
}
