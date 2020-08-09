extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, parse_quote};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as DeriveInput);

    let ident = &inp.ident;
    let ans = if let syn::Data::Struct(ref data) = &inp.data {
        if let syn::Fields::Named(syn::FieldsNamed { ref named, .. }) = data.fields {
            let names: Vec<&Option<syn::Ident>> = named.iter().map(|v| &v.ident).collect();
            let fmts = named.iter().map(|f| {
                if f.attrs.len() == 1 {
                    let meta = f.attrs[0].parse_meta().unwrap();
                    if let syn::Meta::NameValue(ref nv) = meta {
                        if let syn::Lit::Str(ref litstr) = nv.lit {
                            return litstr.value();
                        }
                    }
                }

                "{:?}".to_owned()
            });

            let generics = add_trait_bounds(inp.generics);
            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

            quote! {
                impl #impl_generics ::std::fmt::Debug for #ident #ty_generics #where_clause {
                    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        fmt.debug_struct(stringify!(#ident))#(.field(stringify!(#names), &format_args!(#fmts, &self.#names)))*.finish()
                    }
                }
            }
        } else {
            unimplemented!();
        }
    } else {
        unimplemented!();
    };

    TokenStream::from(ans)
}

fn add_trait_bounds(mut generics: syn::Generics) -> syn::Generics {
    for param in &mut generics.params {
        if let syn::GenericParam::Type(ref mut tp) = param {
            tp.bounds.push(parse_quote!(::std::fmt::Debug));
        }
    }

    generics
}