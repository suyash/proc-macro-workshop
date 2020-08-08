extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as DeriveInput);

    let ident = &inp.ident;
    let ans = if let syn::Data::Struct(ref data) = &inp.data {
        if let syn::Fields::Named(syn::FieldsNamed { ref named, .. }) = data.fields {
            let names: Vec<&Option<syn::Ident>> = named.iter().map(|v| &v.ident).collect();

            quote! {
                struct X {
                    #named
                }

                impl ::std::fmt::Debug for #ident {
                    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        fmt.debug_struct(stringify!(#ident))#(.field(stringify!(#names), &self.#names))*.finish()
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
