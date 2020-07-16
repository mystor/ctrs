use ::core::ops::Not as _;

pub(in crate)
fn generate (
    mod_name: &'_ ::syn::Ident,
) -> ::syn::Result<::proc_macro::TokenStream>
{Ok({
    let syn = crate::error::IoErrorExt::into_syn;
    let ref in_file = format!("src/{}.rs", mod_name);
    debug!(in_file);
    let ref out_file = format!(
        "{OUT_DIR}/inline_proc_macros/{mod_name}.rs",
        OUT_DIR = renv!("OUT_DIR"),
        mod_name = mod_name,
    );
    debug!(out_file);
    if debug!(already_up_to_date(in_file, out_file)).not() {
        let ref input = ::std::fs::read_to_string(in_file).map_err(syn)?;
        let input = if let Ok(it) = input.parse() { it } else {
            // If we are here it means the input file could not even be properly
            // tokenized. At that point, we stop our generation and instead just
            // bundle the input as is, so as to let the compilation process proceed
            // as usual to lead a slightly better spanned error message.
            return Ok(::quote::quote!(
                mod #mod_name;
            ).into());
        };
        let generated = crate::compile_proc_macro::compile(mod_name, input)?;
        ::std::fs::write(out_file, generated.to_string().as_bytes()).map_err(syn)?;
    }
    let ret = ::quote::quote! {
        #[macro_use]
        #[doc(hidden)] pub
        mod #mod_name {
            // The actual expansion
            include!(concat!(
                env!("OUT_DIR"),
                "/inline_proc_macros/",
                stringify!(#mod_name), ".rs",
            ));
            // For #[macro_export] to work, `eval_wasm` needs to be exported as
            // well. Perform the re-export here, to avoid name collisions with
            // other calls to `#[inline_proc::macro_use]`.
            #[doc(hidden)] pub
            use ::inline_proc_macros::__eval_wasm__
                as __inline_proc_macros__eval_wasm__
            ;
            // To make sure a recompilation is triggered when the proc_macro
            // src file is changed (since it technically is never loaded into
            // the main Rust code), we include its contents in a constant.
            const _: &[u8] = include_bytes!(
                concat!(stringify!(#mod_name), ".rs")
            );
        }
    };
    #[cfg(feature = "trace-macros")] {
        println!("\n#[inline_proc::macro_use] expands to:\n");
        crate::utils::log_stream(ret.to_string());
    }
    ret.into()    
})}

/// Main point of outputting the output of the proc_macro into a file.
/// 
/// By accessing the filesystem, we can use the generated file to "cache" the
/// result of compiling the procedural macro code to avoid rerunning it when,
/// for instance, the `src/proc_macros.rs` was left untouched.
fn already_up_to_date (
    in_file: &'_ str,
    out_file: &'_ str,
) -> bool
{
    (|| Ok::<_, ::std::io::Error>(
        ::std::path::Path::metadata(in_file.as_ref())?
            .modified()?
        <
        ::std::path::Path::metadata(out_file.as_ref())?
            .modified()?
    ))().unwrap_or(false)
}
