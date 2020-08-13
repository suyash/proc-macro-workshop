extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Group;
use syn::{Result, Ident, Token, LitInt, parse_macro_input, parse::{ParseStream, Parse}};
use quote::quote;

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

        Ok(SeqInput{
            name,
            start, 
            end,
            body,
        })
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let _ = parse_macro_input!(input as SeqInput);

    let ans = quote!{};

    TokenStream::from(ans)
}
