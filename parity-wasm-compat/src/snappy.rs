// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Use rust native snappy for wasm32


#[cfg(target_arch = "wasm32")]
extern crate snap;

#[cfg(not(target_arch = "wasm32"))]
extern crate parity_snappy;

#[cfg(not(target_arch = "wasm32"))]
pub use parity_snappy as snappy;


#[cfg(target_arch = "wasm32")]
pub mod snappy {

	use std::fmt;

	#[inline]
	pub fn max_compressed_len(len: usize) -> usize {
		snap::max_compress_len(len)
	}
	pub fn decompressed_len(compressed: &[u8]) -> Result<usize, InvalidInput> {
		Ok(snap::decompress_len(compressed)?)
	}
	pub fn compress(input: &[u8]) -> Vec<u8> {
		let mut enc = snap::Encoder::new();
		enc.compress_vec(input).expect("No failure on compression")
	}
	// TODO this proto is not really efficient, snappy compression should use inline buffered
	// see snap writer and reader trait (plus return error)
	pub fn compress_into(input: &[u8], output: &mut Vec<u8>) -> usize {
		let mut enc = snap::Encoder::new();
		let l = max_compressed_len(input.len());
		if output.len() < l {
			output.resize(l,0);
		}
		enc.compress(input, &mut output[..]).expect("No failure on compression")
	}
	pub fn decompress(input: &[u8]) -> Result<Vec<u8>, InvalidInput> {
		let mut dec = snap::Decoder::new();
		Ok(dec.decompress_vec(input)?)
	}
	// TODO this proto is not really efficient, snappy compression should use inline buffered
	// This is bad it build huge buffer
	pub fn decompress_into(input: &[u8], output: &mut Vec<u8>) -> Result<usize, InvalidInput> {
		let mut dec = snap::Decoder::new();
		let l = decompressed_len(input)?;
		if output.len() < l {
			output.resize(l,0);
		}
		Ok(dec.decompress(input, &mut output[..])?)
	}

	#[derive(Debug)]
	pub struct InvalidInput;

	impl std::error::Error for InvalidInput {
		fn description(&self) -> &str {
			"Attempted snappy decompression with invalid input"
		}
	}

	impl fmt::Display for InvalidInput {
		fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
			write!(f, "Attempted snappy decompression with invalid input")
		}
	}

	impl std::convert::From<snap::Error> for InvalidInput {
		fn from(_: snap::Error) -> Self {
			InvalidInput
		}
	}

}
