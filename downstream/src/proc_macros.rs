use proc_macro::{*, TokenTree as TT};

#[proc_macro] pub
fn success (input: TokenStream)
 -> TokenStream
{
    if input.to_string() == "42" {
        TokenStream::new()
    } else {
        let mut span = Span::call_site();
        if let Some(tt) = input.into_iter().next() {
            span = tt.span();
        }
        ::quote::quote_spanned!(span =>
            compile_error!("Expected `42`");
        ).into()
    }
}
