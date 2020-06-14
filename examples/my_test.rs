::inline_proc_macros::compile! {
    use ::proc_macro::TokenStream;
    use syn::*;
    use ::quote::{quote, ToTokens};

    #[proc_macro] pub
    fn my_proc_macro (ts: TokenStream)
      -> TokenStream
    {
        let fname = parse_macro_input!(ts as ItemFn);
        let comment = format!(
            "Got a func! {:?}",
            fname.into_token_stream().to_string(),
        );
        quote! {
            fn main ()
            {
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
