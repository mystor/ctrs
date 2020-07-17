// This gets appended to the proc_macro source code that the users provide:
// `proc_macro2` gets aliases as `proc_macro` as a convenience hack,
// and a `TokenStream2`-based implementation of the convenient
// `parse_macro_input` is also provided.

extern crate proc_macro2 as proc_macro;

#[macro_export]
macro_rules! parse_macro_input {
    (
        $expr:tt as $T:ty
    ) => (
        match ::syn::parse2::<$T>($expr) {
            | Ok(it) => it,
            | Err(err) => return err.to_compile_error()/*.into()*/,
        }
    );

    (
        $expr:expr
    ) => (
        crate::parse_macro_input!($expr as _)
    );
}
