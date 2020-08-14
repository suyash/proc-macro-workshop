extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Group, TokenStream as TokenStream2, TokenTree as TokenTree2};
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

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as SeqInput);
    let stream = &inp.body.stream();

    (inp.start.base10_parse::<u64>().unwrap()..inp.end.base10_parse::<u64>().unwrap())
        .map(|i| expand(stream.clone(), &inp.name, i))
        .collect::<TokenStream2>()
        .into()
}

fn expand(stream: TokenStream2, f: &Ident, index: u64) -> TokenStream2 {
    stream
        .into_iter()
        .map(|tt| expand_tree(tt, f, index))
        .collect()
}

fn expand_tree(tt: TokenTree2, f: &Ident, index: u64) -> TokenTree2 {
    match tt {
        TokenTree2::Group(g) => {
            let exp = Group::new(g.delimiter(), expand(g.stream(), f, index));
            TokenTree2::Group(exp)
        }
        TokenTree2::Ident(ident) if &ident == f => {
            TokenTree2::Literal(proc_macro2::Literal::u64_unsuffixed(index))
        }
        tt => tt,
    }
}
