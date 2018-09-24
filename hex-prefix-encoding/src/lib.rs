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

//! Ethereum hex-prefix encoding (HPE).
//!
//! Encode a slice of bytes in HPE.

use std::iter::once;

/// Hex-prefix Encoding. Encodes a payload and a flag. The high nibble of the first
/// bytes contains the flag; the lowest bit of the flag encodes the oddness of the
/// length and the second-lowest bit encodes whether the node is a value node. The
/// low nibble of the first byte is zero in the case of an even number of nibbles
/// in the payload, otherwise it is set to the first nibble of the payload.
/// All remaining nibbles (now an even number) fit properly into the remaining bytes.
///
/// The "termination marker" and "leaf-node" specifier are equivalent.
///
/// Input nibbles are in range `[0, 0xf]`.
///
/// ```markdown
///  [0,0,1,2,3,4,5]   0x10_01_23_45	// length is odd (7) so the lowest bit of the high nibble of the first byte is `1`; it is not a leaf node, so the second-lowest bit of the high nibble is 0; given it's an odd length, the lower nibble of the first byte is set to the first nibble. All in all we get 0b0001_000 (oddness) + 0b0000_0000 (is leaf?) + 0b0000_0000 = 0b0001_0000 = 0x10 and then we append the other nibbles
///  [0,1,2,3,4,5]     0x00_01_23_45	// length is even (6) and this is not a leaf node so the high nibble of the first byte is 0; the low nibble of the first byte is unused (0)
///  [1,2,3,4,5]       0x11_23_45   	// odd length, not leaf => high nibble of 1st byte is 0b0001 and low nibble of 1st byte is set to the first payload nibble (1) so all in all: 0b00010001, 0x11
///  [0,0,1,2,3,4]     0x00_00_12_34	// even length, not leaf => high nibble is 0 and the low nibble is unused so we get 0x00 and then the payload: 0x00_00_12…
///  [0,1,2,3,4]       0x10_12_34		// odd length, not leaf => oddness flag + first nibble (0) => 0x10
///  [1,2,3,4]         0x00_12_34
///  [0,0,1,2,3,4,5,T] 0x30_01_23_45	// odd length (7), leaf => high nibble of 1st byte is 0b0011; low nibble is set to 1st payload nibble so the first encoded byte is 0b0011_0000, i.e. 0x30
///  [0,0,1,2,3,4,T]   0x20_00_12_34	// even length (6), lead => high nibble of 1st byte is 0b0010; low nibble unused
///  [0,1,2,3,4,5,T]   0x20_01_23_45
///  [1,2,3,4,5,T]     0x31_23_45		// odd length (5), leaf => high nibble of 1st byte is 0b0011; low nibble of 1st byte is set to first payload nibble (1) so the 1st byte becomes 0b0011_0001, i.e. 0x31
///  [1,2,3,4,T]       0x20_12_34
/// ```
pub fn hex_prefix_encode<'a>(nibbles: &'a [u8], leaf: bool) -> impl Iterator<Item = u8> + 'a {
	let inlen = nibbles.len();
	let oddness_factor = inlen % 2;

	let first_byte = {
		let mut bits = ((inlen as u8 & 1) + (2 * leaf as u8)) << 4;
		if oddness_factor == 1 {
			bits += nibbles[0];
		}
		bits
	};
	once(first_byte)
		.chain(nibbles[oddness_factor..]
		.chunks(2)
		.map(|ch| ch[0] << 4 | ch[1]))
}

/// Modified version of HPN that uses the two high bits of the hight nibble to
/// indicate Leaf|Extension, which in combination with the second-lowest bit
/// (aka "termination marker"), lets parity-codec determine the node type. This
/// version of hex_prefix_encode always set the high bit to `1` as we assume
/// that only Leaf|Extension nodes use HPN.
/// (Yes, this is a horrible hack and will be improved upon.)
pub fn hex_prefix_encode_substrate<'a>(nibbles: &'a [u8], leaf: bool) -> impl Iterator<Item = u8> + 'a {
	let inlen = nibbles.len();
	let oddness_factor = inlen % 2;

	let first_byte = {
		let mut bits = (8 + (inlen as u8 & 1) + (2 * leaf as u8)) << 4;
		if oddness_factor == 1 {
			bits += nibbles[0];
		}
		bits
	};
	once(first_byte).chain(nibbles[oddness_factor..].chunks(2).map(|ch| ch[0] << 4 | ch[1]))
}


#[cfg(test)]
mod test_super {
    use super::hex_prefix_encode;

	#[test]
	fn test_hex_prefix_encode() {
		let v = vec![0, 0, 1, 2, 3, 4, 5];
		let e = vec![0x10, 0x01, 0x23, 0x45];
		let h = hex_prefix_encode(&v, false).collect::<Vec<_>>();
		assert_eq!(h, e);

		let v = vec![0, 1, 2, 3, 4, 5];
		let e = vec![0x00, 0x01, 0x23, 0x45];
		let h = hex_prefix_encode(&v, false).collect::<Vec<_>>();
		assert_eq!(h, e);

		let v = vec![0, 1, 2, 3, 4, 5];
		let e = vec![0x20, 0x01, 0x23, 0x45];
		let h = hex_prefix_encode(&v, true).collect::<Vec<_>>();
		assert_eq!(h, e);

		let v = vec![1, 2, 3, 4, 5];
		let e = vec![0x31, 0x23, 0x45];
		let h = hex_prefix_encode(&v, true).collect::<Vec<_>>();
		assert_eq!(h, e);

		let v = vec![1, 2, 3, 4];
		let e = vec![0x00, 0x12, 0x34];
		let h = hex_prefix_encode(&v, false).collect::<Vec<_>>();
		assert_eq!(h, e);

		let v = vec![4, 1];
		let e = vec![0x20, 0x41];
		let h = hex_prefix_encode(&v, true).collect::<Vec<_>>();
		assert_eq!(h, e);
	}
}
