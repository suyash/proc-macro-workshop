extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned, DeriveInput};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let mut inp = parse_macro_input!(input as DeriveInput);

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

            add_trait_bounds(&mut inp.generics, named);

            let (impl_generics, ty_generics, where_clause) = inp.generics.split_for_impl();
            let ident = &inp.ident;

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

fn add_trait_bounds(
    generics: &mut syn::Generics,
    fields: &Punctuated<syn::Field, syn::token::Comma>,
) {
    let where_token = syn::token::Where {
        span: generics.span(),
    };

    for param in &mut generics.params {
        if let syn::GenericParam::Type(ref mut tp) = param {
            if check(&tp.ident, fields) {
                let mut found = false;

                for field in fields {
                    let ty = &field.ty;
                    if let syn::Type::Path(syn::TypePath {
                        path: syn::Path { ref segments, .. },
                        ..
                    }) = ty
                    {
                        if segments.len() == 1 {
                            if let syn::PathSegment {
                                arguments:
                                    syn::PathArguments::AngleBracketed(
                                        syn::AngleBracketedGenericArguments { ref args, .. },
                                    ),
                                ..
                            } = &segments[0]
                            {
                                let arg = &args[0];
                                if let syn::GenericArgument::Type(ref ty) = arg {
                                    if let syn::Type::Path(syn::TypePath {
                                        path: syn::Path { ref segments, .. },
                                        ..
                                    }) = ty
                                    {
                                        if segments.len() > 1 {
                                            let predicates: Punctuated<
                                                syn::WherePredicate,
                                                syn::token::Comma,
                                            > = parse_quote! {
                                                #ty: ::std::fmt::Debug
                                            };

                                            // TODO: this only allows one clause, figure out doing more

                                            generics.where_clause = Some(syn::WhereClause {
                                                where_token,
                                                predicates,
                                            });

                                            found = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if !found {
                    tp.bounds.push(parse_quote!(::std::fmt::Debug));
                }
            }
        }
    }
}

fn check(ident: &syn::Ident, fields: &Punctuated<syn::Field, syn::token::Comma>) -> bool {
    for field in fields {
        let ty = &field.ty;
        if let syn::Type::Path(syn::TypePath {
            path: syn::Path { ref segments, .. },
            ..
        }) = ty
        {
            if segments.len() == 1 {
                let segment = &segments[0];
                let sident = &segment.ident;
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    ref args,
                    ..
                }) = &segment.arguments
                {
                    if args.len() == 1 {
                        let arg = &args[0];
                        if let syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                            path: syn::Path { ref segments, .. },
                            ..
                        })) = arg
                        {
                            if segments.len() == 1 {
                                let segment = &segments[0];
                                if &segment.ident == ident && sident == "PhantomData" {
                                    return false;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    true
}
