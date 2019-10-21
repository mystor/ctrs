# Compile-Time Rust (wip name)

Hack to implement inline procedural macros.

Requres the `wasm32-unknown-unknown` target to be installed.

**note** Extremely experimental!

## Example

```rust
use ctrs::ctrs;

// Declare a "macro crate"
ctrs! {
    macro crate my_test_crate;
    
    // The crates 'proc_macro2', 'syn', and 'quote' can be imported.
    use proc_macro2::TokenStream;
    use syn::*;
    use quote::quote;

    // Macros are marked with 'proc_macro'
    #[proc_macro]
    pub fn my_proc_macro(ts: TokenStream) -> TokenStream {
        let fname = parse2::<ItemFn>(ts).unwrap();
        let comment = format!("Got a func! {:?}", fname);
        quote! {
            fn main() {
                println!("{}", #comment);
            }
        }
    }
}

// And can be invoked like a bang!-macro
my_proc_macro! {
    const fn cool_func<T>(t: T) -> U {
        let x = 10;
        20
    }
}
```

## How does it work?

todo

