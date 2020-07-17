#![cfg_attr(feature = "nightly-docs",
    feature(external_doc),
    doc(include = "../README.md")
)] //!
#![deny(rust_2018_idioms)]
#![deny(missing_docs)]

#[allow(rust_2018_idioms)] extern crate proc_macro; // For retro-compat
use ::proc_macro::TokenStream;
use ::std::ops::Not as _;

#[macro_use]
mod utils;

mod compile_proc_macro;
mod error;
mod eval;
#[path = "macro_use.rs"]
mod macro_use_mod; // otherwise `cargo doc` gets confused

#[proc_macro] #[doc(hidden)] /** Not part of the public API **/ pub
fn __eval_wasm__ (input: TokenStream)
  -> TokenStream
{
    eval::eval_wasm(input)
}


#[cfg_attr(feature = "nightly-docs",
    doc(include = "macro_use.md"),
)] ///
#[proc_macro_attribute] pub
fn macro_use (attrs: TokenStream, input: TokenStream)
  -> TokenStream
{
    struct Input(::syn::Ident); impl ::syn::parse::Parse for Input {
        fn parse (input: ::syn::parse::ParseStream<'_>)
          -> ::syn::Result<Self>
        {
            let ::syn::ItemMod { ident, content, .. } = input.parse()?;
            if let Some((_, items)) = content {
                if items.is_empty().not() {
                    return Err(::syn::Error::new_spanned(
                        ::quote::quote!(#(#items)*), // span over all items
                        "Expected empty body",
                    ));
                }
            }
            Ok(Input(ident))
        }
    }

    let _: ::syn::parse::Nothing = ::syn::parse_macro_input!(attrs);
    let Input(ref mod_name) = ::syn::parse_macro_input!(input);
    match macro_use_mod::generate(mod_name) {
        | Ok(it) => it,
        | Err(err) => err.to_compile_error().into(),
    }
}
