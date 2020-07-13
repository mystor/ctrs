pub(in crate)
trait IoErrorExt {
    fn into_syn (self: Self)
      -> ::syn::Error
    ;
}
impl IoErrorExt
    for ::std::io::Error
{
    fn into_syn (self: Self)
      -> ::syn::Error
    {
        ::syn::Error::new(::proc_macro2::Span::call_site(), self.to_string())
    }
}
