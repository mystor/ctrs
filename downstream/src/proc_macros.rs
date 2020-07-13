use ::proc_macro2::TokenStream;
use ::syn::{*,
    parse::{Parser, ParseStream},
};
use ::quote::quote;

#[macro_export]
#[proc_macro] pub
fn success (_: TokenStream)
 -> TokenStream
{
    TokenStream::new()
}

macro_rules! unwrap {($expr:expr) => (
    match $expr {
        | Ok(it) => it,
        | Err(err) => return err.to_compile_error().into(),
    }
)}

#[proc_macro] pub
fn impl_arrays (input: TokenStream)
 -> TokenStream
{
    let (start, end) = unwrap!((|input: ParseStream<'_>| Ok({
        let start: usize = input.parse::<LitInt>().and_then(|it| it.base10_parse())?;
        let _: Token![..] = input.parse()?;
        let end: usize = input.parse::<LitInt>().and_then(|it| it.base10_parse())?;
        (start, end)
    })).parse2(input));
    let mut ret = TokenStream::new();
    #[allow(nonstandard_style)]
    for N in start .. end {
        ret.extend(quote! {
            impl<T> Array
                for [T; #N]
            {
                type Elem = T;

                const LEN: usize = #N;

                #[inline]
                fn as_slice (self: &'_ Self)
                  -> &'_ [T]
                {
                    self
                }

                #[inline]
                fn as_slice_mut (self: &'_ mut Self)
                  -> &'_ mut [T]
                {
                    self
                }
            }
        });
    }
    ret.into()
}
