extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::{TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, Error, Item};

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);

    match item {
        Item::Enum(itemenum) => {
            let stream = quote! { #itemenum };
            TokenStream::from(stream)
        }
        _ => {
            let err = Error::new_spanned(
                TokenStream2::from(args),
                "expected enum or match expression",
            );
            err.to_compile_error().into()
        }
    }
}
