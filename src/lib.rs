// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

extern crate proc_macro;

mod de;
mod en;

use de::{impl_decodable, impl_decodable_wrapper};
use en::{impl_encodable, impl_encodable_wrapper};
use proc_macro::TokenStream;

#[proc_macro_derive(RlpEncodable, attributes(rlp))]
pub fn encodable(input: TokenStream) -> TokenStream {
	let ast = syn::parse(input).unwrap();
	let gen = impl_encodable(&ast);
	gen.into()
}

#[proc_macro_derive(RlpEncodableWrapper)]
pub fn encodable_wrapper(input: TokenStream) -> TokenStream {
	let ast = syn::parse(input).unwrap();
	let gen = impl_encodable_wrapper(&ast);
	gen.into()
}

#[proc_macro_derive(RlpDecodable, attributes(rlp))]
pub fn decodable(input: TokenStream) -> TokenStream {
	let ast = syn::parse(input).unwrap();
	let gen = impl_decodable(&ast);
	gen.into()
}

#[proc_macro_derive(RlpDecodableWrapper)]
pub fn decodable_wrapper(input: TokenStream) -> TokenStream {
	let ast = syn::parse(input).unwrap();
	let gen = impl_decodable_wrapper(&ast);
	gen.into()
}
