#![doc = include_str!("../README.md")]

use generate::ImplConfig;
use proc_macro::TokenStream as TokenStream1;
use syn::{parse_macro_input, ItemImpl};

mod generate;

/// See the [module documentation in the `cuicui_chirp`][doc] crate documentation.
///
/// [doc]: <https://docs.rs/cuicui_chirp/latest/cuicui_chirp/parse_dsl_impl/index.html>
#[proc_macro_attribute]
pub fn parse_dsl_impl(attrs: TokenStream1, input: TokenStream1) -> TokenStream1 {
    let mut config = ImplConfig::default();

    let config_parser = syn::meta::parser(|meta| config.parse(meta));
    parse_macro_input!(attrs with config_parser);

    let mut input = parse_macro_input!(input as ItemImpl);
    generate::parse_dsl_impl(&mut config, &mut input).into()
}
