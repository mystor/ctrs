use ::std::{
    env,
    // fs,
    // io::{self, Write},
    // iter,
    // process::{Command, Stdio},
    // ops::Not as _,
};

extern crate proc_macro;
use ::proc_macro::{
    TokenStream,
};
// use ::proc_macro2::{
//     Span,
//     TokenStream as TokenStream2,
//     TokenTree as TT,
// };
// use ::quote::{
//     quote,
//     quote_spanned,
//     ToTokens,
// };
// use ::syn::{*,
//     parse::{Parse, Parser, ParseStream},
//     punctuated::Punctuated,
//     spanned::Spanned,
// };

// type Result<Ok, Err = ::syn::Error> = ::core::result::Result<Ok, Err>;

fn log_stream (ts: &TokenStream)
{
    let in_str = ts.to_string();
    if in_str.len() > 1000 {
        let pre = in_str.chars().take(400).collect::<String>();
        let post = in_str.chars().rev().take(400).collect::<String>().chars().rev().collect::<String>();
        println!("{} [.. {} chars ..] {}", pre, in_str.len() - 800, post)
    } else {
        println!("{}", in_str);
    }
}

#[proc_macro]
#[doc(hidden)] /** Not part of the public API **/ pub
fn __eval_wasm__ (input: TokenStream)
  -> TokenStream
{
    let debug = env::var("DEBUG_INLINE_MACROS").ok().map_or(false, |s| s == "1");
    if debug {
        println!("<<<__eval_wasm__! {{");
        log_stream(&input);
        println!("}}\n>>>");
    }
    let mut tokens = TokenStream::into_iter(input.into());
    let func =
        tokens
            .next()
            .expect("Missing procmacro name")
            .to_string()
    ;
    let wasm_lit =
        tokens
            .next()
            .expect("Missing WASM-compiled procmacro source code")
            .to_string()
    ;
    assert!(wasm_lit.starts_with('"') && wasm_lit.ends_with('"'));
    let wasm =
        ::base64::decode(&wasm_lit[1 .. wasm_lit.len() - 1])
            .unwrap()
    ;
    ::watt::proc_macro(&func, TokenStream::into(tokens.collect()), &wasm)
}
