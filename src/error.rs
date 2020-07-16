pub(in crate)
trait SynErrExt {
    type Ok;
    fn syn_err (self: Self)
      -> Result<Self::Ok, ::syn::Error>
    ;
}
impl<Ok> SynErrExt
    for Result<Ok, ::std::io::Error>
{
    type Ok = Ok;
    fn syn_err (self: Self)
      -> Result<Ok, ::syn::Error>
    {
        self.map_err(|err| {
          ::syn::Error::new(::proc_macro2::Span::call_site(), err.to_string())
        })
    }
}
