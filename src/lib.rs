extern crate proc_macro;
use ::proc_macro::TokenStream;

#[macro_use]
mod utils;

mod compile_proc_macro;
mod error;
mod eval;
mod macro_use;

#[proc_macro] #[doc(hidden)] /** Not part of the public API **/ pub
fn __eval_wasm__ (input: TokenStream)
  -> TokenStream
{
    eval::eval_wasm(input)
}

#[proc_macro_attribute] #[doc(hidden)] /** Not part of the public API **/ pub
fn macro_use (attrs: TokenStream, input: TokenStream)
  -> TokenStream
{
    let _: ::syn::parse::Nothing = ::syn::parse_macro_input!(attrs);
    let input: ::syn::ItemMod = ::syn::parse_macro_input!(input);
    match macro_use::generate(&input.ident) {
        | Ok(it) => it,
        | Err(err) => err.to_compile_error().into(),
    }
}
