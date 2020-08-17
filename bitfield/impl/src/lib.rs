extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, parse_quote, ExprPath, Fields, FieldsNamed, Ident, ItemStruct, LitInt, Path,
    Type, TypePath,
};

#[proc_macro_attribute]
pub fn bitfield(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);

    let vis = &item.vis;
    let ident = &item.ident;

    let sizes: Vec<ExprPath> = if let Fields::Named(FieldsNamed { ref named, .. }) = &item.fields {
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

    let names: Vec<&Ident> = if let Fields::Named(FieldsNamed { ref named, .. }) = &item.fields {
        named.iter().map(|f| f.ident.as_ref().unwrap()).collect()
    } else {
        unimplemented!()
    };

    let starts: Vec<Vec<TokenStream2>> = (0..sizes.len())
        .map(|i| sizes.iter().take(i).map(|v| v.to_token_stream()).collect())
        .map(|mut v: Vec<TokenStream2>| {
            v.push(quote! {0});
            v
        })
        .collect();

    let ends: Vec<Vec<TokenStream2>> = (0..sizes.len())
        .map(|i| {
            sizes
                .iter()
                .take(i + 1)
                .map(|v| v.to_token_stream())
                .collect()
        })
        .collect();

    let get_names = names
        .iter()
        .map(|name| Ident::new(format!("get_{}", name).as_str(), name.span()));
    let set_names = names
        .iter()
        .map(|name| Ident::new(format!("set_{}", name).as_str(), name.span()));

    let stream = quote! {
        #vis struct #ident {
            data: [u8; (#(#sizes)+*) / 8],
        }

        impl #ident {
            pub fn new() -> #ident {
                #ident {
                    data: [0; (#(#sizes)+*) / 8]
                }
            }

            #(pub fn #get_names(&self) -> u64 {
                let start = #(#starts)+*;
                let end = #(#ends)+*;

                self.get(start, end)
            })*

            #(pub fn #set_names(&mut self, v: u64) {
                let start = #(#starts)+*;
                let end = #(#ends)+*;
                let size = #sizes;

                self.set(v, start, end, size)
            })*

            fn get(&self, start: usize, end: usize) -> u64 {
                let si = start / 8;
                let ei = end / 8;

                let sp = start % 8;
                let ep = end % 8;

                if si == ei {
                    (((1u64 << (8 - sp)) - 1) & self.data[si] as u64) >> (8 - ep)
                } else {
                    let mut ans = ((1u64 << (8 - sp)) - 1) & self.data[si] as u64;
                    for ix in (si + 1)..ei {
                        ans = (ans << 8) | self.data[ix] as u64;
                    }

                    if ei < self.data.len() {
                        ans = (ans << ep) | (self.data[ei] as u64 >> (8 - ep));
                    }

                    ans
                }
            }

            fn set(&mut self, v: u64, start: usize, end: usize, size: usize) {
                let si = start / 8;
                let ei = end / 8;

                let sp = start % 8;
                let ep = end % 8;

                if si == ei {
                    self.data[si] = Self::merge(self.data[si] as u64, sp, ep, v) as u8;
                } else {
                    self.data[si] = Self::merge(self.data[si] as u64, sp, 8, v >> (size - (8 - sp))) as u8;
                    let mut d = 8 - sp;

                    for ix in (si + 1)..ei {
                        let m = (1 << (size - d)) - 1;
                        let m2 = ((1 << 8) - 1) << (size - (d + 8));
                        let v2 = v & m;
                        let v3 = v2 & m2;
                        let v4 = v3 >> (size - (d + 8));
                        self.data[ix] = v4 as u8;
                        d += 8;
                    }

                    if ep > 0 {
                        self.data[ei] = (((1 << ep) - 1) & v) as u8;
                    }
                }
            }

            fn merge(v: u64, start: usize, end: usize, w: u64) -> u64 {
                let x = (((1u64 << start) - 1) << (8 - start)) & v;
                let m = w << (8 - end);
                let y = ((1 << (8 - end)) - 1) & v;
                x | m | y
            }
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
