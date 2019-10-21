use ctrs::ctrs;

ctrs! {
    macro crate my_test_crate;
    use proc_macro2::TokenStream;
    use syn::*;
    use quote::quote;

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

my_proc_macro! {
    const fn cool_func<T>(t: T) -> U {
        let x = 10;
        20
    }
}
