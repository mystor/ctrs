//! Internal implementation crate for `ctrs`

use proc_macro2::TokenStream;
use syn::*;

#[no_mangle]
pub extern "C" fn ctrs(input: TokenStream) -> TokenStream {
    proc_macro2::set_wasm_panic_hook();
    input
}

#[no_mangle]
pub extern "C" fn ctrs2(input: TokenStream) -> TokenStream {
    proc_macro2::set_wasm_panic_hook();
    input
}
