extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, Fields, FieldsNamed, Ident, ItemStruct, LitInt, Path, Type,
    TypePath,
};

#[proc_macro_attribute]
pub fn bitfield(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);

    let vis = &item.vis;
    let ident = &item.ident;

    let types: Vec<Type> = if let Fields::Named(FieldsNamed { ref named, .. }) = &item.fields {
        named
            .iter()
            .map(|f| {
                if let Type::Path(TypePath {
                    path: Path { ref segments, .. },
                    ..
                }) = &f.ty
                {
                    let i = &segments[0].ident;
                    parse_quote! { <#i as Specifier>::BITS }
                } else {
                    unimplemented!()
                }
            })
            .collect()
    } else {
        unimplemented!()
    };

    let stream = quote! {
        #vis struct #ident {
            data: [u8; (#(#types)+*) / 8],
        }
    };

    TokenStream::from(stream)
}

#[proc_macro]
pub fn bitfield_type(input: TokenStream) -> TokenStream {
    let val = parse_macro_input!(input as LitInt);
    let name = format!("B{}", val);
    let ident = Ident::new(name.as_str(), val.span());

    let stream = quote! {
        pub struct #ident;

        impl crate::Specifier for #ident {
            const BITS: usize = #val;
        }
    };

    stream.into()
}
