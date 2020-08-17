extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    spanned::Spanned,
    Data, DeriveInput, Error, ExprPath, Fields, FieldsNamed, Ident, ItemStruct, LitInt, Path,
    Result, Token, Type, TypePath,
};

#[proc_macro_attribute]
pub fn bitfield(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);

    let vis = &item.vis;
    let ident = &item.ident;

    let types: Vec<&Type> = if let Fields::Named(FieldsNamed { ref named, .. }) = &item.fields {
        named.iter().map(|f| &f.ty).collect()
    } else {
        unimplemented!()
    };

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

    let hold_types: Vec<Type> = if let Fields::Named(FieldsNamed { ref named, .. }) = &item.fields {
        named
            .iter()
            .map(|f| {
                if let Type::Path(TypePath {
                    path: Path { ref segments, .. },
                    ..
                }) = &f.ty
                {
                    let i = &segments[0].ident;
                    parse_quote! { <#i as Specifier>::HoldType }
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

        impl bitfield::checks::CheckTotalSizeIsMultipleOfEightBits for #ident {
            type Size = std::marker::PhantomData<[(); (#(#sizes)+*) % 8]>;
        }

        impl #ident {
            pub fn new() -> #ident {
                #ident {
                    data: [0; (#(#sizes)+*) / 8]
                }
            }

            #(pub fn #get_names(&self) -> #hold_types {
                let start = #(#starts)+*;
                let end = #(#ends)+*;

                #types::to_hold(self.get(start, end))
            })*

            #(pub fn #set_names(&mut self, v: #hold_types) {
                let start = #(#starts)+*;
                let end = #(#ends)+*;
                let size = #sizes;

                self.set(#types::from_hold(v), start, end, size)
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

struct TypeParams {
    size: LitInt,
    ty: Type,
}

impl Parse for TypeParams {
    fn parse(input: ParseStream) -> Result<Self> {
        let size: LitInt = input.parse()?;
        let _: Token![,] = input.parse()?;
        let ty: Type = input.parse()?;

        Ok(TypeParams { size, ty })
    }
}

#[proc_macro]
pub fn bitfield_type(input: TokenStream) -> TokenStream {
    let params = parse_macro_input!(input as TypeParams);

    let val = &params.size;
    let ty = &params.ty;

    let name = format!("B{}", val);
    let ident = Ident::new(name.as_str(), val.span());

    let stream = quote! {
        pub struct #ident;

        impl crate::Specifier for #ident {
            const BITS: usize = #val;
            type HoldType = #ty;

            fn to_hold(v: u64) -> Self::HoldType {
                v as Self::HoldType
            }

            fn from_hold(v: Self::HoldType) -> u64 {
                v as u64
            }
        }
    };

    stream.into()
}

#[proc_macro_derive(BitfieldSpecifier)]
pub fn derive(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as DeriveInput);

    if let DeriveInput {
        ref ident,
        data: Data::Enum(ref data),
        ..
    } = &inp
    {
        let len = data.variants.len();

        if !len.is_power_of_two() {
            let stream = Error::new(
                Span::call_site(),
                "BitfieldSpecifier expected a number of variants which is a power of 2",
            );
            return TokenStream::from(stream.to_compile_error());
        }

        let bits = len.trailing_zeros() as usize;

        let variant_conditions: Vec<TokenStream2> = data
            .variants
            .iter()
            .map(|v| {
                let aident = &v.ident;
                parse_quote!(if #ident::#aident as u64 == v { return #ident::#aident; })
            })
            .collect();

        let variants: Vec<TokenStream2> = data.variants.iter().map(|v| {
            let vident = &v.ident;
            let span = v.span();
            quote_spanned!{ span => impl bitfield::checks::CheckDiscriminantInRange<[(); #ident::#vident as usize]> for #ident {
                    type Type = std::marker::PhantomData<[(); ((#ident::#vident as usize) < (1 << #bits)) as usize]>;
                }
            }
        }).collect();

        let ans = quote! {
            impl bitfield::Specifier for #ident {
                const BITS: usize = #bits;
                type HoldType = #ident;

                fn to_hold(v: u64) -> Self::HoldType {
                    #(#variant_conditions)*

                    unimplemented!()
                }

                fn from_hold(v: Self::HoldType) -> u64 {
                    v as u64
                }
            }

            #(#variants)*
        };

        TokenStream::from(ans)
    } else {
        unimplemented!()
    }
}
