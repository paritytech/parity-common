// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::*;

#[test]
fn basic_test() {
	let mut dest = [0; 32];
	let salt = [5; 32];
	let secret = [7; 32];
	sha256(3, Salt(&salt[..]), Secret(&secret[..]), &mut dest);
	let res = [
		242, 33, 31, 124, 36, 223, 179, 185, 206, 175, 190, 253, 85, 33, 23, 126, 141, 29, 23, 97, 66, 63, 51, 196, 27,
		255, 135, 206, 74, 137, 172, 87,
	];
	assert_eq!(res, dest);
}
