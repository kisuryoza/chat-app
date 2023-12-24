use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn expand(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = input.ident;
    let data = match input.data {
        syn::Data::Struct(ref data) => data,
        syn::Data::Enum(_) => unimplemented!("enum"),
        syn::Data::Union(_) => unimplemented!("union"),
    };
    let funcs = match data.fields {
        syn::Fields::Named(ref fields) => handle_named_fields(fields),
        syn::Fields::Unnamed(_) => unimplemented!("unnamed fields"),
        syn::Fields::Unit => unimplemented!("unit"),
    };

    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #funcs
        }
    };

    Ok(expanded)
}

fn handle_named_fields(fields: &syn::FieldsNamed) -> TokenStream {
    let recurse = fields.named.iter().map(|f| {
        let name = &f.ident;
        let (ret_type, body) = match &f.ty {
            syn::Type::Path(type_path) => named_fields_with_path(name.as_ref(), type_path),
            syn::Type::Reference(type_ref) => (quote! { #type_ref }, quote! { self.#name }),
            _ => unimplemented!("field types"),
        };

        quote! {
            pub fn #name(&self) -> #ret_type {
                #body
            }
        }
    });
    quote! { #(#recurse)* }
}

fn named_fields_with_path(
    name: Option<&proc_macro2::Ident>,
    ty: &syn::TypePath,
) -> (TokenStream, TokenStream) {
    let type_path = &ty.path;

    let last_segment = type_path.segments.last().unwrap();
    match &last_segment.arguments {
        syn::PathArguments::None => {
            let ty = match last_segment.ident.to_string().as_str() {
                "String" => syn::parse_str::<syn::Path>("str").unwrap(),
                _ => type_path.clone(),
            };

            let ret_type = quote! { & #ty };
            let body = quote! { & self.#name };

            (ret_type, body)
        }
        // The `<'a, T>` in `std::slice::iter<'a, T>`.
        syn::PathArguments::AngleBracketed(path_arg) => {
            let last_arg = path_arg.args.last().unwrap();
            let ret_type = match last_segment.ident.to_string().as_str() {
                "Vec" => quote! { & [ #last_arg ] },
                _ => quote! { & #last_arg },
            };
            let body = quote! { self.#name.as_ref() };

            (ret_type, body)
        }
        // The `(A, B) -> C` in `Fn(A, B) -> C`.
        syn::PathArguments::Parenthesized(_) => {
            unimplemented!("parenthesized args")
        }
    }
}
