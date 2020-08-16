use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Error, Item};

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);

    match item {
        Item::Enum(itemenum) => {
            let variants = &itemenum.variants;

            if !variants.is_empty() {
                let mut v = vec![variants.first().unwrap()];
                for val in variants.iter().skip(1) {
                    if &v.last().unwrap().ident > &val.ident {
                        for vv in v.iter() {
                            if &vv.ident > &val.ident {
                                let msg = format!("{} should sort before {}", &val.ident, &vv.ident);
                                let err = Error::new_spanned(&val.ident, msg.as_str());

                                let base = quote!{ #itemenum };
                                let curr = err.to_compile_error();

                                return vec![base, curr].into_iter().collect::<TokenStream2>().into();
                            }
                        }
                    }

                    v.push(val);
                }
            }

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
