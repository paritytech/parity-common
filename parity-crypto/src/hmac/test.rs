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


use super::*;
#[test]
fn simple_mac_and_verify() {
	let input = b"Some bytes";
	let big_input = vec![7u8;2000];

	let key1 = vec![3u8;64];
	let key2 = vec![4u8;128];

	let sig_key1 = SigKey::sha256(&key1[..]);
	let sig_key2 = SigKey::sha512(&key2[..]);

	let mut signer1 = Signer::with(&sig_key1);
	let mut signer2 = Signer::with(&sig_key2);

	signer1.update(&input[..]);
	for i in 0 .. big_input.len() / 33 {
		signer2.update(&big_input[i*33..(i+1)*33]);
	}
	signer2.update(&big_input[(big_input.len() / 33)*33..]);
	let sig1 = signer1.sign();
	assert_eq!(&sig1[..], [223, 208, 90, 69, 144, 95, 145, 180, 56, 155, 78, 40, 86, 238, 205, 81, 160, 245, 88, 145, 164, 67, 254, 180, 202, 107, 93, 249, 64, 196, 86, 225]);
	let sig2 = signer2.sign();
	assert_eq!(&sig2[..], &[29, 63, 46, 122, 27, 5, 241, 38, 86, 197, 91, 79, 33, 107, 152, 195, 118, 221, 117, 119, 84, 114, 46, 65, 243, 157, 105, 12, 147, 176, 190, 37, 210, 164, 152, 8, 58, 243, 59, 206, 80, 10, 230, 197, 255, 110, 191, 180, 93, 22, 255, 0, 99, 79, 237, 229, 209, 199, 125, 83, 15, 179, 134, 89][..]);
	assert_eq!(&sig1[..], &sign(&sig_key1, &input[..])[..]);
	assert_eq!(&sig2[..], &sign(&sig_key2, &big_input[..])[..]);
	let verif_key1 = VerifyKey::sha256(&key1[..]);
	let verif_key2 = VerifyKey::sha512(&key2[..]);
	assert!(verify(&verif_key1, &input[..], &sig1[..]));
	assert!(verify(&verif_key2, &big_input[..], &sig2[..]));
}
