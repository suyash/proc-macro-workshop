use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as DeriveInput);
    let name = inp.ident;
    let buildername = format_ident!("{}Builder", name);
    
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
        ..
    }) = inp.data
    {
        named
    } else {
        unimplemented!();
    };

    let fnames = fields.iter().map(|f| &f.ident).collect::<Vec<&Option<syn::Ident>>>();
    let ftypes = fields.iter().map(|f| &f.ty).collect::<Vec<&syn::Type>>();

    let expanded = quote! {
        use std::error::Error;

        impl #name {
            pub fn builder() -> #buildername {
                #buildername {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }

        pub struct #buildername {
            #(#fnames: Option<#ftypes>),*
        }

        impl #buildername {
            #(fn #fnames(&mut self, #fnames: #ftypes) -> &mut Self {
                self.#fnames = Some(#fnames);
                self
            })*

            pub fn build(&mut self) -> Result<#name, Box<dyn Error>> {
                Ok(#name {
                    executable: self.executable.take().ok_or("Fail")?,
                    args: self.args.take().ok_or("Fail")?,
                    env: self.env.take().ok_or("Fail")?,
                    current_dir: self.current_dir.take().ok_or("Fail")?,
                })
            }
        }
    };

    TokenStream::from(expanded)
}
