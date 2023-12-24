use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn expand(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let name = input.ident;
    let data = match input.data {
        syn::Data::Struct(ref data) => data,
        syn::Data::Enum(_) => unimplemented!("enum"),
        syn::Data::Union(_) => unimplemented!("union"),
    };
    let (params, names) = match data.fields {
        syn::Fields::Named(ref fields) => new_on_named_fields(fields),
        syn::Fields::Unnamed(_) => unimplemented!("unnamed fields"),
        syn::Fields::Unit => unimplemented!("unit"),
    };

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn new(#params) -> Self {
                Self { #names }
            }
        }
    };

    Ok(expanded)
}

fn new_on_named_fields(fields: &syn::FieldsNamed) -> (TokenStream, TokenStream) {
    let params = fields.named.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            #name: #ty,
        }
    });
    let names = fields.named.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name,
        }
    });
    (quote! { #(#params)* }, quote! { #(#names)* })
}
