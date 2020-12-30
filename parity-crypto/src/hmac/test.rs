// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::*;
use hex_literal::hex;

#[test]
fn simple_mac_and_verify() {
	let input = b"Some bytes";
	let big_input = vec![7u8; 2000];

	let key1 = vec![3u8; 64];
	let key2 = vec![4u8; 128];

	let sig_key1 = SigKey::sha256(&key1[..]);
	let sig_key2 = SigKey::sha512(&key2[..]);

	let mut signer1 = Signer::with(&sig_key1);
	let mut signer2 = Signer::with(&sig_key2);

	signer1.update(&input[..]);
	for i in 0..big_input.len() / 33 {
		signer2.update(&big_input[i * 33..(i + 1) * 33]);
	}
	signer2.update(&big_input[(big_input.len() / 33) * 33..]);
	let sig1 = signer1.sign();
	assert_eq!(
		&sig1[..],
		[
			223, 208, 90, 69, 144, 95, 145, 180, 56, 155, 78, 40, 86, 238, 205, 81, 160, 245, 88, 145, 164, 67, 254,
			180, 202, 107, 93, 249, 64, 196, 86, 225
		]
	);
	let sig2 = signer2.sign();
	assert_eq!(
		&sig2[..],
		&[
			29, 63, 46, 122, 27, 5, 241, 38, 86, 197, 91, 79, 33, 107, 152, 195, 118, 221, 117, 119, 84, 114, 46, 65,
			243, 157, 105, 12, 147, 176, 190, 37, 210, 164, 152, 8, 58, 243, 59, 206, 80, 10, 230, 197, 255, 110, 191,
			180, 93, 22, 255, 0, 99, 79, 237, 229, 209, 199, 125, 83, 15, 179, 134, 89
		][..]
	);
	assert_eq!(&sig1[..], &sign(&sig_key1, &input[..])[..]);
	assert_eq!(&sig2[..], &sign(&sig_key2, &big_input[..])[..]);
	let verif_key1 = VerifyKey::sha256(&key1[..]);
	let verif_key2 = VerifyKey::sha512(&key2[..]);
	assert!(verify(&verif_key1, &input[..], &sig1[..]));
	assert!(verify(&verif_key2, &big_input[..], &sig2[..]));
}

fn check_test_vector(key: &[u8], data: &[u8], expected_256: &[u8], expected_512: &[u8]) {
	// Sha-256
	let sig_key = SigKey::sha256(&key);
	let mut signer = Signer::with(&sig_key);
	signer.update(&data);
	let signature = signer.sign();
	assert_eq!(&signature[..], expected_256);
	assert_eq!(&signature[..], &sign(&sig_key, data)[..]);
	let ver_key = VerifyKey::sha256(&key);
	assert!(verify(&ver_key, data, &signature));

	// Sha-512
	let sig_key = SigKey::sha512(&key);
	let mut signer = Signer::with(&sig_key);
	signer.update(&data);
	let signature = signer.sign();
	assert_eq!(&signature[..], expected_512);
	assert_eq!(&signature[..], &sign(&sig_key, data)[..]);
	let ver_key = VerifyKey::sha512(&key);
	assert!(verify(&ver_key, data, &signature));
}

#[test]
fn ietf_test_vectors() {
	// Test vectors from https://tools.ietf.org/html/rfc4231.html#section-4

	// Test Case 1
	check_test_vector(
		&hex!("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b"),
		&hex!("4869205468657265"),
		&hex!(
			"
			b0344c61d8db38535ca8afceaf0bf12b
			881dc200c9833da726e9376c2e32cff7"
		),
		&hex!(
			"
			87aa7cdea5ef619d4ff0b4241a1d6cb0
			2379f4e2ce4ec2787ad0b30545e17cde
			daa833b7d6b8a702038b274eaea3f4e4
			be9d914eeb61f1702e696c203a126854"
		),
	);

	// Test Case 2
	check_test_vector(
		&hex!("4a656665"),
		&hex!("7768617420646f2079612077616e7420666f72206e6f7468696e673f"),
		&hex!(
			"
			5bdcc146bf60754e6a042426089575c7
			5a003f089d2739839dec58b964ec3843"
		),
		&hex!(
			"
			164b7a7bfcf819e2e395fbe73b56e0a3
			87bd64222e831fd610270cd7ea250554
			9758bf75c05a994a6d034f65f8f0e6fd
			caeab1a34d4a6b4b636e070a38bce737"
		),
	);
	// Test Case 3
	check_test_vector(
		&hex!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
		&hex!("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"),
		&hex!(
			"
			773ea91e36800e46854db8ebd09181a7
			2959098b3ef8c122d9635514ced565fe"
		),
		&hex!(
			"
			fa73b0089d56a284efb0f0756c890be9
			b1b5dbdd8ee81a3655f83e33b2279d39
			bf3e848279a722c806b485a47e67c807
			b946a337bee8942674278859e13292fb"
		),
	);

	// Test Case 4
	check_test_vector(
		&hex!("0102030405060708090a0b0c0d0e0f10111213141516171819"),
		&hex!(
			"
			cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd
			cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd
			cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd
			cdcd"
		),
		&hex!(
			"
			82558a389a443c0ea4cc819899f2083a
			85f0faa3e578f8077a2e3ff46729665b"
		),
		&hex!(
			"
			b0ba465637458c6990e5a8c5f61d4af7
			e576d97ff94b872de76f8050361ee3db
			a91ca5c11aa25eb4d679275cc5788063
			a5f19741120c4f2de2adebeb10a298dd"
		),
	);

	// Test Case 6
	check_test_vector(
		&hex!(
			"
			aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
			aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
			aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
			aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
			aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
			aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
			aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
			aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
			aaaaaa"
		),
		&hex!(
			"
			54657374205573696e67204c61726765
			72205468616e20426c6f636b2d53697a
			65204b6579202d2048617368204b6579
			204669727374"
		),
		&hex!(
			"
			60e431591ee0b67f0d8a26aacbf5b77f
			8e0bc6213728c5140546040f0ee37f54"
		),
		&hex!(
			"
			80b24263c7c1a3ebb71493c1dd7be8b4
			9b46d1f41b4aeec1121b013783f8f352
			6b56d037e05f2598bd0fd2215d6a1e52
			95e64f73f63f0aec8b915a985d786598"
		),
	);

	// Test Case 7
	check_test_vector(
		&hex!(
			"
			aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
			aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
			aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
			aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
			aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
			aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
			aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
			aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
			aaaaaa"
		),
		&hex!(
			"
			54686973206973206120746573742075
			73696e672061206c6172676572207468
			616e20626c6f636b2d73697a65206b65
			7920616e642061206c61726765722074
			68616e20626c6f636b2d73697a652064
			6174612e20546865206b6579206e6565
			647320746f2062652068617368656420
			6265666f7265206265696e6720757365
			642062792074686520484d414320616c
			676f726974686d2e"
		),
		&hex!(
			"
			9b09ffa71b942fcb27635fbcd5b0e944
			bfdc63644f0713938a7f51535c3a35e2"
		),
		&hex!(
			"
			e37b6a775dc87dbaa4dfa9f96e5e3ffd
			debd71f8867289865df5a32d20cdc944
			b6022cac3c4982b10d5eeb55c3e4de15
			134676fb6de0446065c97440fa8c6a58"
		),
	);
}
