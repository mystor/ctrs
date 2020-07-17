use ::core::ops::Not as _;

/// Main function driving the code generation.
/// 
/// It proceeds as follows: since knowing the exact file path location is
/// not possible in stable Rust (it would require extracting
/// `SourceFile`-related information from the `Span`s), we restrict ourselves
/// to top-level modules, leading to `src/{mod_name}.rs` paths (everything more
/// fancy (non `src/lib.rs` entry point, nested top-level modules, _etc.) is
/// thus not supported).
/// 
/// Then, the basic idea would be to compile the procedural macro code as
/// standalone executable bytecode, and, by having a helper procedural macro
/// able to read and execute such bytecode, emit `macro_rules!` macros of the
/// form:
/// 
/// ```rust,ignore
/// macro_rules! some_macro_name { ( $($input:tt)* ) => (
///     eval_bytecode! {
///         "...compiled bytecode for the crate..."
///         "some_macro_name" // (a crate may define multiple macros)
///         $($input)*
///     }
/// )}
/// ```
/// 
/// However, in practice, any changes to the downstream crate would trigger a
/// re-evaluation of the proc-macro (since Rust makes no guarantees _w.r.t._
/// caching those), which, in turn, would trigger a re-compilation of the
/// procedural macro, which would lead to things like `cargo watch -x check`
/// becoming annoyingly laggy.
/// 
/// So, we have to go down a slightly more complex path: that of generating
/// the `macro_rules!` definition within an extra / generated source file that
/// we can then use as cache (using `mtime` metadata).
/// 
/// This means we are relying on procedural macros interacting with the
/// filesystem in a persistent manner. This is indeed susceptible to breakage
/// in some situations, such as:
/// 
///   - compiling a dep exporting macros, and then moving the
///     whole directory containing both the downstream crate and that lib (which
///     is unlikely, hence the decision to sacrifice that situation to speed up
///     the most usual case),
/// 
///   - or just Rust deciding to sandbox the execution of a procedural macro,
///     but at that point, we should have wasm virtual machines bundled within
///     Rust we should make bundling inline proc_macros as a built-in feature
///     of `cargo / rustc` become trivial, thus making this whole 
///     obsolete.
/// 
/// And since we are already using the FS, the compiled bytecode can also be
/// kept bundled within its own file too, avoiding the cost of
/// encoding / decoding it into / from a string literal.
/// 
/// Finally, regarding the choice of the single-file bundling + executor engine,
/// between the two most sensible options (native dynamic libraries, or WASM
/// plugins), the latter has been chosen, mainly for its simplicity
/// (portability-wise), and the added sandboxing security does not hurt either.
/// 
/// ---
/// 
/// Thus, the actual emitted code becomes:
/// 
///  1. generating a `path/to/generated/compiled_bytecode.wasm` file;
/// 
///  2. generating a `path/to/generated/some_module.rs` file somewhere,
///     which contains:
/// 
///     ```rust
///     macro_rules! some_macro_name { ( $($input:tt)* ) => (
///         eval_wasm! {
///             "path/to/generated/compiled_bytecode.wasm"
///             "some_macro_name"
///             $($input)*
///         }
///     )}
/// 
///  3. And finally, have the main procedural macro expand to
///     `include!("path/to/generated/some_module.rs")`,
///     using an `mtime` comparison between `src/some_module.rs` and this file
///     to know when we can skip the steps `1.` and `2.`
pub(in crate)
fn generate (
    mod_name: &'_ ::syn::Ident,
) -> ::syn::Result<::proc_macro::TokenStream>
{Ok({
    use ::proc_macro2::TokenStream as TokenStream2;
    use crate::error::SynErrExt;

    let ref in_file = format!(
        "{CARGO_MANIFEST_DIR}/src/{mod_name}.rs",
        CARGO_MANIFEST_DIR = renv!("CARGO_MANIFEST_DIR"),
        mod_name = mod_name,
    );
    debug!(in_file);
    let ref out_file = format!(
        "{OUT_DIR}/inline_proc_macros/{mod_name}.rs",
        OUT_DIR = renv!("OUT_DIR"),
        mod_name = mod_name,
    );
    debug!(out_file);
    if debug!(already_up_to_date(in_file, out_file)).not() {
        let ref input = ::std::fs::read_to_string(in_file).syn_err()?;
        let input = if let Ok(it) = input.parse::<TokenStream2>() { it } else {
            // If we are here it means the input file could not even be properly
            // tokenized. At that point, we stop our generation and instead just
            // bundle the input as is, so as to let the compilation process
            // proceed as usual, to lead a slightly better spanned error message.
            return Ok(::quote::quote!(
                mod #mod_name;
            ).into());
        };
        // `generated` will contain the different `macro_rules!` definitions.
        let generated: TokenStream2 =
            crate::compile_proc_macro::compile(mod_name, input)?
        ;
        ::std::fs::write(out_file, generated.to_string().as_bytes()).syn_err()?;
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
            // other calls to `#[inline_proc_macro::macro_use]`.
            #[doc(hidden)] /** Not part of the public API **/ pub
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
        println!("\n<<<\n#[inline_proc_macro::macro_use]\n=== yields ===");
        crate::utils::log_stream(ret.to_string());
        println!("\n>>>\n");
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
