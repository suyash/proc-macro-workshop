extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{
    Delimiter, Group, Literal, TokenStream as TokenStream2, TokenTree as TokenTree2,
};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitInt, Result, Token,
};

#[derive(Debug)]
struct SeqInput {
    name: Ident,
    start: u64,
    end: u64,
    body: Group,
}

impl Parse for SeqInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let _: Token![in] = input.parse()?;
        let start: LitInt = input.parse()?;
        let _: Token![..] = input.parse()?;

        let res: Result<Token![=]> = input.parse();

        let end: LitInt = input.parse()?;
        let body: Group = input.parse()?;

        let start = start.base10_parse::<u64>().unwrap();
        let mut end = end.base10_parse::<u64>().unwrap();

        if res.is_ok() {
            end += 1;
        }

        Ok(SeqInput {
            name,
            start,
            end,
            body,
        })
    }
}

impl SeqInput {
    fn rep_block(&self, group: &Group) -> (TokenStream2, bool) {
        let mut iter = group.stream().into_iter();

        let mut last1 = None;
        let mut last2 = None;

        let mut repeated = false;

        let mut v = Vec::new();

        loop {
            match iter.next() {
                None => break,
                Some(tt) => {
                    match &tt {
                        TokenTree2::Group(ref g) => {
                            let (pg, pr) = self.rep_block(g);
                            repeated = repeated || pr;
                            let ng = Group::new(g.delimiter(), pg);

                            if last2.is_some() {
                                v.push(last2.unwrap());
                            }

                            last2 = last1;
                            last1 = Some(TokenTree2::Group(ng));
                            continue;
                        }

                        TokenTree2::Punct(ref punct) if punct.as_char() == '*' => {
                            if last2.is_some() && last1.is_some() {
                                match (last2.as_ref().unwrap(), last1.as_ref().unwrap()) {
                                    (
                                        TokenTree2::Punct(ref punct2),
                                        TokenTree2::Group(ref group),
                                    ) if punct2.as_char() == '#'
                                        && group.delimiter() == Delimiter::Parenthesis =>
                                    {
                                        repeated = true;

                                        let pt = (self.start..self.end)
                                            .map(|i| self.expand(group.stream(), i))
                                            .collect::<TokenStream2>();

                                        v.pop();
                                        v.pop();
                                        v.push(TokenTree2::Group(Group::new(Delimiter::None, pt)));

                                        last2 = None;
                                        last1 = None;

                                        continue;
                                    }
                                    _ => {}
                                }
                            }
                        }

                        _ => {}
                    }

                    if last2.is_some() {
                        v.push(last2.unwrap());
                    }

                    last2 = last1;
                    last1 = Some(tt);
                }
            }
        }

        if last2.is_some() {
            v.push(last2.unwrap());
        }

        if last1.is_some() {
            v.push(last1.unwrap());
        }

        (v.into_iter().collect(), repeated)
    }

    fn expand(&self, stream: TokenStream2, index: u64) -> TokenStream2 {
        let mut iter = stream.into_iter();
        let mut v = Vec::new();

        let mut last2 = None;
        let mut last1 = None;

        loop {
            match iter.next() {
                None => break,
                Some(tt) => {
                    let rtt = match &tt {
                        TokenTree2::Group(ref group) => TokenTree2::Group(Group::new(
                            group.delimiter(),
                            self.expand(group.stream(), index),
                        )),
                        TokenTree2::Ident(ref ident) => {
                            if ident == &self.name {
                                TokenTree2::Literal(Literal::u64_unsuffixed(index))
                            } else {
                                TokenTree2::Ident(ident.clone())
                            }
                        }
                        tt => tt.clone(),
                    };

                    if last1.is_none() {
                        last1 = Some(rtt);
                        continue;
                    }

                    if let TokenTree2::Ident(ref oident) = &tt {
                        if oident == &self.name {
                            if last2.is_some() && last1.is_some() {
                                let ll2 = last2.as_ref().unwrap();
                                let ll1 = last1.as_ref().unwrap();

                                match (ll2, ll1) {
                                    (
                                        TokenTree2::Ident(ref ident),
                                        TokenTree2::Punct(ref punct),
                                    ) if punct.as_char() == '#' => {
                                        let idname = format!("{}{}", ident, index);
                                        v.push(TokenTree2::Ident(Ident::new(
                                            idname.as_str(),
                                            ident.span(),
                                        )));

                                        last2 = None;
                                        last1 = None;

                                        continue;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    if last2.is_some() {
                        v.push(last2.unwrap());
                    }

                    last2 = last1;
                    last1 = Some(rtt);
                }
            }
        }

        if last2.is_some() {
            v.push(last2.unwrap());
        }

        if last1.is_some() {
            v.push(last1.unwrap());
        }

        v.into_iter().collect()
    }
}

impl Into<TokenStream2> for SeqInput {
    fn into(self) -> TokenStream2 {
        let (ts, repeated) = self.rep_block(&self.body);
        if repeated {
            ts
        } else {
            (self.start..self.end)
                .map(|i| self.expand((&self.body).stream(), i))
                .collect::<TokenStream2>()
        }
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as SeqInput);
    let ans: TokenStream2 = inp.into();
    TokenStream::from(ans)
}
