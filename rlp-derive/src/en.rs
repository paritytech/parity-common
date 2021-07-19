// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use proc_macro2::TokenStream;
use quote::quote;

pub fn impl_encodable(ast: &syn::DeriveInput) -> TokenStream {
	let body = if let syn::Data::Struct(s) = &ast.data {
		s
	} else {
		panic!("#[derive(RlpEncodable)] is only defined for structs.");
	};

	let stmts: Vec<_> = body
		.fields
		.iter()
		.enumerate()
		.map(|(i, field)| encodable_field(i, field))
		.collect();
	let name = &ast.ident;

	let stmts_len = stmts.len();
	let stmts_len = quote! { #stmts_len };
	let impl_block = quote! {
		impl rlp::Encodable for #name {
			fn rlp_append(&self, stream: &mut rlp::RlpStream) {
				stream.begin_list(#stmts_len);
				#(#stmts)*
			}
		}
	};

	quote! {
		const _: () = {
			extern crate rlp;
			#impl_block
		};
	}
}

pub fn impl_encodable_wrapper(ast: &syn::DeriveInput) -> TokenStream {
	let body = if let syn::Data::Struct(s) = &ast.data {
		s
	} else {
		panic!("#[derive(RlpEncodableWrapper)] is only defined for structs.");
	};

	let stmt = {
		let fields: Vec<_> = body.fields.iter().collect();
		if fields.len() == 1 {
			let field = fields.first().expect("fields.len() == 1; qed");
			encodable_field(0, field)
		} else {
			panic!("#[derive(RlpEncodableWrapper)] is only defined for structs with one field.")
		}
	};

	let name = &ast.ident;

	let impl_block = quote! {
		impl rlp::Encodable for #name {
			fn rlp_append(&self, stream: &mut rlp::RlpStream) {
				#stmt
			}
		}
	};

	quote! {
		const _: () = {
			extern crate rlp;
			#impl_block
		};
	}
}

fn encodable_field(index: usize, field: &syn::Field) -> TokenStream {
	let ident = if let Some(ident) = &field.ident {
		quote! { #ident }
	} else {
		let index = syn::Index::from(index);
		quote! { #index }
	};

	let id = quote! { self.#ident };

	if let syn::Type::Path(path) = &field.ty {
		let top_segment = path.path.segments.first().expect("there must be at least 1 segment");
		let ident = &top_segment.ident;
		if ident == "Vec" {
			let inner_ident = {
				if let syn::PathArguments::AngleBracketed(angle) = &top_segment.arguments {
					if let syn::GenericArgument::Type(syn::Type::Path(path)) =
						angle.args.first().expect("Vec has only one angle bracketed type; qed")
					{
						&path.path.segments.first().expect("there must be at least 1 segment").ident
					} else {
						panic!("rlp_derive not supported");
					}
				} else {
					unreachable!("Vec has only one angle bracketed type; qed")
				}
			};
			quote! { stream.append_list::<#inner_ident, _>(&#id); }
		} else {
			quote! { stream.append(&#id); }
		}
	} else {
		panic!("rlp_derive not supported");
	}
}
