use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Error, Item, ItemFn, ExprMatch, Pat, visit_mut::{self, VisitMut}};

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

struct Check {
    err: Option<Error>
}

impl VisitMut for Check {
    fn visit_expr_match_mut(&mut self, node: &mut ExprMatch) {
        if node.attrs.len() == 1 {
            node.attrs.clear();

            // TODO: check that the attribute is sorted

            if node.arms.len() > 1 {
                let mut v = vec![&node.arms[0].pat];

                'outer: for (ix, arm) in node.arms.iter().skip(1).enumerate() {
                    let pat = &arm.pat;

                    match (v.last().unwrap(), pat) {
                        (Pat::TupleStruct(ref first), Pat::TupleStruct(ref second)) => {
                            let s1 = &first.path.segments;
                            let s2 = &second.path.segments;

                            let mut cs1 = "".to_string();
                            let mut cs2 = "".to_string();

                            for (i1, i2) in s1.iter().zip(s2.iter()) {
                                let ident1 = &i1.ident;
                                let ident2 = &i2.ident;

                                cs1 = format!("{}::{}", cs1, ident1);
                                cs2 = format!("{}::{}", cs2, ident2);

                                if ident1 > ident2 {
                                    let msg = format!("{} should sort before {}", &cs2[2..], &cs1[2..]);
                                    let err = Error::new_spanned(&s2, msg.as_str());
                                    self.err = Some(err);
                                    break 'outer;
                                }
                            }
                        },
                        (Pat::Ident(ref first), Pat::Ident(ref second)) => {
                            if first.ident > second.ident {
                                let msg = format!("{} should sort before {}", first.ident, second.ident);
                                let err = Error::new_spanned(&second.ident, msg.as_str());
                                self.err = Some(err);
                                break 'outer;
                            }
                        }
                        (_, Pat::Wild(ref pat)) => {
                            if ix != node.arms.len() - 2 {
                                let err = Error::new_spanned(&pat, "Expected Wildcard to appear at the end");
                                self.err = Some(err);
                                break 'outer;
                            }
                        }
                        _ => {
                            println!("{:?}", pat);
                            let err = Error::new_spanned(&v.last().unwrap(), "unsupported by #[sorted]");
                            self.err = Some(err);
                            break 'outer;
                        }
                    }

                    v.push(&pat);
                }
            }
        }
        visit_mut::visit_expr_match_mut(self, node);
    }
}

#[proc_macro_attribute]
pub fn check(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut itemfn = parse_macro_input!(input as ItemFn);

    let mut checker = Check { err: None };
    checker.visit_item_fn_mut(&mut itemfn);

    let stream = quote!{#itemfn};

    match checker.err {
        Some(err) => {
            let stream: TokenStream2 = vec![stream, err.to_compile_error()].into_iter().collect();
            stream.into()
        },
        None => stream.into(),
    }
}
