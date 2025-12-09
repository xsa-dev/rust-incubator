use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, Token, parse_macro_input};

struct MapEntries {
    pairs: Punctuated<MapEntry, Token![,]>,
}

impl Parse for MapEntries {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let pairs = if input.is_empty() {
            Punctuated::new()
        } else {
            Punctuated::parse_terminated(input)?
        };

        Ok(Self { pairs })
    }
}

struct MapEntry {
    key: Expr,
    value: Expr,
}

impl Parse for MapEntry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let key: Expr = input.parse()?;
        input.parse::<Token![=>]>()?;
        let value: Expr = input.parse()?;

        Ok(Self { key, value })
    }
}

#[proc_macro]
pub fn btreemap(tokens: TokenStream) -> TokenStream {
    let entries = parse_macro_input!(tokens as MapEntries);

    if entries.pairs.is_empty() {
        return quote!(::std::collections::BTreeMap::new()).into();
    }

    let inserts = entries.pairs.iter().map(|entry| {
        let MapEntry { key, value } = entry;
        quote! {
            map.insert(#key, #value);
        }
    });

    TokenStream::from(quote! {{
        let mut map = ::std::collections::BTreeMap::new();
        #(#inserts)*
        map
    }})
}
