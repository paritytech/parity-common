// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

//! A crate for deriving the MallocSizeOf trait.
//!
//! This is a copy of Servo malloc_size_of_derive code, modified to work with
//! our `parity_util_mem` library

extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate synstructure;

#[cfg(not(test))]
decl_derive!([MallocSizeOf, attributes(ignore_malloc_size_of)] => malloc_size_of_derive);

fn malloc_size_of_derive(s: synstructure::Structure) -> proc_macro2::TokenStream {
	let match_body = s.each(|binding| {
		let ignore = binding.ast().attrs.iter().any(|attr| match attr.parse_meta().unwrap() {
			syn::Meta::Path(ref path) | syn::Meta::List(syn::MetaList { ref path, .. })
				if path.is_ident("ignore_malloc_size_of") =>
			{
				panic!(
					"#[ignore_malloc_size_of] should have an explanation, \
					 e.g. #[ignore_malloc_size_of = \"because reasons\"]"
				);
			}
			syn::Meta::NameValue(syn::MetaNameValue { ref path, .. }) if path.is_ident("ignore_malloc_size_of") => true,
			_ => false,
		});
		if ignore {
			None
		} else if let syn::Type::Array(..) = binding.ast().ty {
			Some(quote! {
				for item in #binding.iter() {
					sum += parity_util_mem::MallocSizeOf::size_of(item, ops);
				}
			})
		} else {
			Some(quote! {
				sum += parity_util_mem::MallocSizeOf::size_of(#binding, ops);
			})
		}
	});

	let ast = s.ast();
	let name = &ast.ident;
	let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
	let mut where_clause = where_clause.unwrap_or(&parse_quote!(where)).clone();
	for param in ast.generics.type_params() {
		let ident = &param.ident;
		where_clause.predicates.push(parse_quote!(#ident: parity_util_mem::MallocSizeOf));
	}

	let tokens = quote! {
		impl #impl_generics parity_util_mem::MallocSizeOf for #name #ty_generics #where_clause {
			#[inline]
			#[allow(unused_variables, unused_mut, unreachable_code)]
			fn size_of(&self, ops: &mut parity_util_mem::MallocSizeOfOps) -> usize {
				let mut sum = 0;
				match *self {
					#match_body
				}
				sum
			}
		}
	};

	tokens
}
