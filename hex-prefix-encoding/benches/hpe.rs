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

#![feature(test)]

extern crate test;
extern crate hex_prefix_encoding as hpe;

use test::Bencher;
use hpe::hex_prefix_encode;

#[bench]
fn hex_prefix_encoding(b: &mut Bencher) {
	let alfabet = b"c9491bae672ce3ffee05e80b8f53edc0c6861d568dc178a031d4224f2b07ed4543086757f428a98287b6dcd796e0e74825ffe71d313515f755dcfbd21cb95d74dd1a97a7117bf828dfb47351e2767ddf6408afd31d07d54df34cab486c64c1715d767f7d3181e27a20ccc93a7160de51305ad4b8916f1cc1888e64bd38219e513278ed671c3240c3783e93eef60d3c701a6c5eb1d18f8233038dc5f86531d76c864bcab17675aa69f8662d185bf3a3b61408dfa6e4ae63712f76bb3b9bad281aae5b68129250d0b7dc1f13eeefc0d563f0548c3d56aa33b748f303f9f336a653fe83c77d6b0ed6e0cefc50846368d6ba5834a92588c05688ddd1c33146e45472743e3a1cee9cc84fe7edbd870b776586e3787aa1cb8ff6fae500903cea58308acf20e84f9f54351b1a5eefbe69805f50223cc9a973599f63941ccdeec670f9c00e0b24f31c6754114a722a9ec32cea5ad9879971a0054df341000481bbe6717c087263248872e8509f10b342a06d37b8ec6f08e29fbc05f6fc140514fc0a1c2f6b611e6043b5665094594ba14b976255";
	let d = &alfabet[4..198];
	let d2 = &alfabet[37..310];
	assert!(d.len() % 2 == 0);
	assert!(d2.len() % 2 == 1);
	b.iter(|| {
		let _ = hex_prefix_encode(d.clone(), true);
		let _ = hex_prefix_encode(d.clone(), false);
		let _ = hex_prefix_encode(d2.clone(), true);
		let _ = hex_prefix_encode(d2.clone(), false);
	})
}
