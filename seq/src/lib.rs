extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Group, Literal, TokenStream as TokenStream2, TokenTree as TokenTree2};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitInt, Result, Token,
};

#[derive(Debug)]
struct SeqInput {
    name: Ident,
    start: LitInt,
    end: LitInt,
    body: Group,
}

impl Parse for SeqInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let _: Token![in] = input.parse()?;
        let start: LitInt = input.parse()?;
        let _: Token![..] = input.parse()?;
        let end: LitInt = input.parse()?;
        let body: Group = input.parse()?;

        Ok(SeqInput {
            name,
            start,
            end,
            body,
        })
    }
}

impl SeqInput {
    fn expand(&self, stream: TokenStream2, index: u64) -> TokenStream2 {
        let mut iter = stream.into_iter();
        let mut v = Vec::new();

        let mut ids = None;
        let mut idp = None;

        loop {
            match iter.next() {
                None => break,
                Some(tt) => match tt {
                    TokenTree2::Group(g) => {
                        if ids.is_some() {
                            v.push(TokenTree2::Ident(ids.unwrap()));
                            ids = None;
                        }

                        if idp.is_some() {
                            v.push(TokenTree2::Punct(idp.unwrap()));
                            idp = None;
                        }

                        v.push(TokenTree2::Group(Group::new(
                            g.delimiter(),
                            self.expand(g.stream(), index),
                        )));
                    }
                    TokenTree2::Ident(ident) => {
                        if &ident == &self.name && ids.is_some() && idp.is_some() {
                            let s = format!("{}{}", ids.unwrap(), index);
                            v.push(TokenTree2::Ident(Ident::new(s.as_str(), ident.span())));

                            ids = None;
                            idp = None;
                            
                            continue;
                        }

                        if &ident == &self.name {
                            v.push(TokenTree2::Literal(Literal::u64_unsuffixed(index)));
                        } else {
                            if ids.is_some() {
                                v.push(TokenTree2::Ident(ids.unwrap()));
                            }

                            ids = Some(ident);
                        }
                    }
                    TokenTree2::Punct(punct) => {
                        if punct.as_char() == '#' && ids.is_some() && idp.is_none() {
                            idp = Some(punct);
                        } else {
                            if ids.is_some() {
                                v.push(TokenTree2::Ident(ids.unwrap()));
                                ids = None;
                            }

                            if idp.is_some() {
                                v.push(TokenTree2::Punct(idp.unwrap()));
                                idp = None;
                            }

                            v.push(TokenTree2::Punct(punct));
                        }
                    }
                    tt => v.push(tt),
                },
            }
        }

        v.into_iter().collect()
    }
}

impl Into<TokenStream2> for SeqInput {
    fn into(self) -> TokenStream2 {
        let start = self.start.base10_parse::<u64>().unwrap();
        let end = self.end.base10_parse::<u64>().unwrap();

        (start..end)
            .map(|i| self.expand(self.body.stream(), i))
            .collect::<TokenStream2>()
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as SeqInput);
    let ans: TokenStream2 = inp.into();
    TokenStream::from(ans)
}
