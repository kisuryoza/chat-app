extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod get;
mod new;

/// Expands a struct
///
/// ```ignore
///#[derive(New)]
///struct Name {
///    a: Type,
///    ...
///}
///impl<'_> Name<'_> {
///    pub fn new(a: &'_ Type, ...) -> Self {
///        Self { a, ... }
///    }
///}
/// ```
#[proc_macro_derive(New)]
pub fn derive_new(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    new::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Expands a struct
///
/// ```ignore
///#[derive(Get)]
///struct Name {
///    a: Type,
///    ...
///}
///impl<'_> Name<'_> {
///    pub fn a(&self) -> &Type {
///        &self.a
///    }
///    ...
///}
/// ```
#[proc_macro_derive(Get)]
pub fn derive_get(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    get::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
