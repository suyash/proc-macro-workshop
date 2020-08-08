use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

fn internal_type<'a>(args: &'a syn::PathArguments) -> ::std::option::Option<&'a syn::Type> {
    if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
        ref args,
        ..
    }) = args
    {
        let arg = args.first().unwrap();
        if let syn::GenericArgument::Type(ref ity) = arg {
            return ::std::option::Option::Some(ity);
        }
    }

    ::std::option::Option::None
}

#[proc_macro_derive(Builder, attributes(builder))]
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

    let builder_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;

        if let syn::Type::Path(syn::TypePath {
            path: syn::Path { ref segments, .. },
            ..
        }) = ty
        {
            if segments.len() == 1 {
                let segment = segments.first().unwrap();
                let tident = &segment.ident;
                let args = &segment.arguments;
                if tident == "Option" {
                    if let syn::PathArguments::AngleBracketed(
                        syn::AngleBracketedGenericArguments { ref args, .. },
                    ) = args
                    {
                        let arg = args.first().unwrap();
                        if let syn::GenericArgument::Type(ref ity) = arg {
                            return quote! { #ident: ::std::option::Option<#ity> };
                        } else {
                            unimplemented!();
                        }
                    } else {
                        unimplemented!();
                    }
                }
            }
        }

        quote! { #ident: ::std::option::Option<#ty> }
    });

    let fns = fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        let attrs = &f.attrs;

        if let syn::Type::Path(syn::TypePath {
            path: syn::Path { ref segments, .. },
            ..
        }) = ty
        {
            if segments.len() == 1 {
                let tident = &segments[0].ident;
                let args = &segments[0].arguments;
                if tident == "Option" {
                    let ity = internal_type(args).unwrap();
                    return quote! {
                        fn #ident(&mut self, #ident: #ity) -> &mut Self {
                            self.#ident = ::std::option::Option::Some(#ident);
                            self
                        }
                    };
                }

                if tident == "Vec" && attrs.len() == 1 {
                    let att = attrs.first().unwrap();
                    let meta = att.parse_meta().unwrap();
                    if let syn::Meta::List(ref ml) = meta {
                        let nested = &ml.nested;
                        if nested.len() == 1 {
                            if let syn::NestedMeta::Meta(syn::Meta::NameValue(ref nv)) = nested[0] {
                                let path = &nv.path;
                                let litstr = match &nv.lit {
                                    syn::Lit::Str(ref s) => s,
                                    _ => unimplemented!(),
                                };
                                if path.segments.len() == 1 {
                                    let segident = &(path.segments[0]).ident;
                                    if segident == "each" {
                                        let name = litstr.value();
                                        let name = name.as_str();
                                        let iident = syn::Ident::new(name, litstr.span());
                                        let ity = internal_type(args).unwrap();
                                        return quote! {
                                            fn #iident(&mut self, #iident: #ity) -> &mut Self {
                                                self.#ident = match self.#ident.take() {
                                                    ::std::option::Option::None => ::std::option::Option::Some(vec![#iident]),
                                                    ::std::option::Option::Some(mut v) => {
                                                        v.push(#iident);
                                                        ::std::option::Option::Some(v)
                                                    }
                                                };
                                                self
                                            }
                                        };
                                    } else {
                                        let err = syn::Error::new_spanned(
                                            ml,
                                            "expected `builder(each = \"...\")`",
                                        );
                                        return err.to_compile_error();
                                    }
                                } else {
                                    unimplemented!();
                                }
                            } else {
                                unimplemented!();
                            }
                        }
                    } else {
                        unimplemented!();
                    }
                }
            }
        }

        quote! {
            fn #ident(&mut self, #ident: #ty) -> &mut Self {
                self.#ident = ::std::option::Option::Some(#ident);
                self
            }
        }
    });

    let build_args = fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;

        if let syn::Type::Path(syn::TypePath {
            path: syn::Path { ref segments, .. },
            ..
        }) = ty
        {
            if segments.len() == 1 {
                let segment = segments.first().unwrap();
                let tident = &segment.ident;
                if tident == "Option" {
                    return quote! {
                        #ident: self.#ident.take()
                    };
                }

                if tident == "Vec" && f.attrs.len() == 1 {
                    return quote! {
                        #ident: match (self.#ident.take()) {
                            ::std::option::Option::None => vec![],
                            ::std::option::Option::Some(v) => v
                        }
                    };
                }
            }
        }

        quote! {
            #ident: self.#ident.take().ok_or("Fail")?
        }
    });

    let build_init = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! { #ident: ::std::option::Option::None }
    });

    let expanded = quote! {
        impl #name {
            pub fn builder() -> #buildername {
                #buildername {
                    #(#build_init),*
                }
            }
        }

        pub struct #buildername {
            #(#builder_fields),*
        }

        impl #buildername {
            #(#fns)*

            pub fn build(&mut self) -> ::std::result::Result<#name, ::std::boxed::Box<dyn ::std::error::Error>> {
                Ok(#name {
                    #(#build_args),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
