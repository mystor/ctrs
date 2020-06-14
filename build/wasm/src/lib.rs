//! Internal implementation crate for `ctrs`

use ::proc_macro2::{
    TokenStream,
    TokenTree,
};
use ::quote::{
    quote,
    quote_spanned,
    ToTokens,
};
use ::syn::{*,
    parse::{Parse, Parser, ParseStream},
    punctuated::Punctuated,
};

struct BuildResult {
    _name: Ident,
    wasm: TokenTree,
    macros: Punctuated<Ident, Token![,]>,
}

impl Parse for BuildResult {
    fn parse(stream: ParseStream) -> Result<Self> {
        Ok(BuildResult {
            _name: stream.parse()?,
            wasm: stream.parse()?,
            macros: stream.parse_terminated(Ident::parse)?,
        })
    }
}

// #[no_mangle]
pub(in crate)
// extern "C"
fn build_result(input: TokenStream) -> TokenStream {
    // proc_macro2::set_wasm_panic_hook();

    let input = syn::parse2::<BuildResult>(input).unwrap();

    let wasm = &input.wasm;
    let mut result = TokenStream::new();
    for macro_name in &input.macros {
        result.extend(quote! {
            macro_rules! #macro_name {
                ($($t:tt)*) => {
                    ::ctrs::ctrs! { __eval_wasm__ #macro_name #wasm $($t)* }
                };
            }
        });
    }
    result
}

#[derive(Debug)]
struct CtrsInput {
    name: Ident,
    items: Vec<Item>,
}

impl Parse for CtrsInput {
    fn parse(stream: ParseStream) -> Result<Self> {
        stream.parse::<Token![macro]>()?;
        stream.parse::<Token![crate]>()?;
        let name = stream.parse::<Ident>()?;
        stream.parse::<Token![;]>()?;

        let mut items = <Vec<Item>>::new();
        while !stream.is_empty() {
            items.push(stream.parse()?);
        }

        Ok(CtrsInput {name, items})
    }
}

// #[no_mangle]
pub(in crate)
// extern "C"
fn ctrs(input: TokenStream) -> TokenStream {
    // proc_macro2::set_wasm_panic_hook();

    let mut input = syn::parse2::<CtrsInput>(input).unwrap();

    // WOO Let's do some sketchy transformations~
    let mut macros = <Punctuated<Ident, Token![,]>>::new();
    for item in &mut input.items {
        match item {
            Item::Fn(func) => {
                // Check for the `proc_macro` attribute, and remove it.
                let old_len = func.attrs.len();
                func.attrs.retain(|attr| !attr.path.is_ident("proc_macro"));
                if old_len > func.attrs.len() {
                    // Record our macro
                    // FIXME: Record vis here too?
                    macros.push(func.sig.ident.clone());

                    // Transform the method into a wasm export.
                    func.attrs.push(parse_quote!(#[no_mangle]));
                    func.sig.abi = Some(parse_quote!(extern "C"));
                    // func.block.stmts.insert(0, parse_quote!(::proc_macro2::set_wasm_panic_hook();));
                    func.vis = parse_quote!(pub);
                }
            }
            _ => {}
        }
    }

    let name = &input.name;
    let items = &input.items;
    quote! {
        ::ctrs::ctrs! { __build_wasm__ #name { #(#items)* } #macros }
    }
}
