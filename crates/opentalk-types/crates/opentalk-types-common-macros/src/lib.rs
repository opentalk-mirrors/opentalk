// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Macros for opentalk-types-common.

#![deny(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

use opentalk_types_common_identifiers::{
    asset_file_kind::AssetFileKind, feature_id::FeatureId, module_id::ModuleId,
};
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{LitStr, parse_macro_input};

fn generate_const_id<T: std::str::FromStr + ToString>(
    input: TokenStream,
    path: TokenStream2,
) -> TokenStream
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let Ok(crate_name) = crate_name("opentalk-types-common") else {
        return quote! { compile_error!("Couldn't find opentalk-types-common crate") }.into();
    };

    let crate_name = match crate_name {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = proc_macro2::Ident::new(&name, Span::call_site());
            quote!(#ident)
        }
    };

    let input = parse_macro_input!(input as LitStr);
    let value = input.value();

    match value.parse::<T>() {
        Ok(value) => {
            let value = value.to_string();
            quote! {
                #crate_name::#path::__new_borrowed(#value)
            }
        }
        Err(e) => {
            let msg = e.to_string();
            quote! { compile_error!(#msg) }
        }
    }
    .into()
}

/// Create a constant `ModuleId` at compile time.
#[proc_macro]
pub fn module_id(input: TokenStream) -> TokenStream {
    generate_const_id::<ModuleId>(input, quote!(modules::ModuleId))
}

/// Create a constant `FeatureId` at compile time.
#[proc_macro]
pub fn feature_id(input: TokenStream) -> TokenStream {
    generate_const_id::<FeatureId>(input, quote!(features::FeatureId))
}

/// Create a constant `AssetFileKind` at compile time.
#[proc_macro]
pub fn asset_file_kind(input: TokenStream) -> TokenStream {
    generate_const_id::<AssetFileKind>(input, quote!(assets::AssetFileKind))
}
