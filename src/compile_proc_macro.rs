use ::std::{
    env,
    fs,
    io::{self, Write},
    process::{Command, Stdio},
    ops::Not as _,
};

use ::proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use ::quote::{
    quote,
    quote_spanned,
};
use ::syn::{*,
    spanned::Spanned,
};
use ::tempdir::{
    TempDir,
};

type Result<Ok, Err = ::syn::Error> = ::core::result::Result<Ok, Err>;

fn ignore<T, E> (_: Result<T, E>)
{}

fn sip_hash (it: &'_ (impl ::core::hash::Hash + ?Sized))
 -> u64
{
    #[allow(deprecated)]
    let ref mut hasher = ::core::hash::SipHasher::new();
    it.hash(hasher);
    ::core::hash::Hasher::finish(hasher)
}

/// Invoke rustc to build a `wasm32-unknown-unknown` crate with dependencies on
/// `unicode_xid`, `proc_macro2`, `syn`, and `quote`.
fn compile_to_wasm (source: &'_ str)
  -> io::Result<String>
{
    define_strings! {
        const WASM_TARGET = "wasm32-unknown-unknown";
        const CRATE_NAME = "inline_proc_macros";
    }
    ignore(rustup_ensure_has_target(WASM_TARGET));
    // Build within a tempdir
    let tmp = TempDir::new("inline_proc_macros_tempdir")?;
    let tmp_path =
        tmp .path()
            .to_str()
            .expect("`TempDir` generated a non-UTF-8 path")
    ;
    let wasm_path = format!(
        "{out_dir}/inline_proc_macro_{hash:016x}.wasm",
        out_dir = renv!("OUT_DIR"),
        hash = sip_hash(source),
    );
    let mut cmd = Command::new(renv!("RUSTC"));
    cmd.args(&[
        "-", // input source code is piped
        "-o", &wasm_path,
        "--target", WASM_TARGET,
        "--edition", "2018",
        "--crate-type", "cdylib",
        "--crate-name", CRATE_NAME,
        "-L", &["dependency=", tmp_path].concat(),
        "--color=always",
    ]);
    macro_rules! rlibs {(
        $($lib:ident),* $(,)?
    ) => ({
        struct Paths {
            $(
                $lib: String,
            )*
        }
        let paths = Paths {
            $(
                $lib:
                    tmp .path()
                        .join(concat!("lib", stringify!($lib), ".rlib"))
                        .to_string_lossy()
                        .into_owned()
                ,
            )*
        };
        $(
            fs::write(&paths.$lib, &include_bytes! {
                concat!(
                    env!("OUT_DIR"),
                    "/wasm32-unknown-unknown/release/",
                    "lib", stringify!($lib), ".rlib",
                )
            }[..])?;
            cmd.arg("--extern");
            cmd.arg([stringify!($lib), "=", &paths.$lib].concat());
        )*
    })}
    rlibs! {
        proc_macro2, quote, syn, unicode_xid,
    }

    // Spawn the compiler
    let mut child = cmd.stdin(Stdio::piped()).spawn()?;
    // Pipe the source code in (scoped binding to ensure pipe is closed).
    match child.stdin.take().unwrap() { mut stdin => {
        stdin.write_all(source.as_bytes())?;
        stdin.write_all(stringify!(
            extern crate proc_macro2 as proc_macro;

            #[macro_export]
            macro_rules! parse_macro_input {
                (
                    $expr:tt as $T:ty
                ) => (
                    match ::syn::parse2::<$T>($expr) {
                        | Ok(it) => it,
                        | Err(err) => return err.to_compile_error().into(),
                    }
                );

                (
                    $expr:expr
                ) => (
                    parse_macro_input!($expr as _)
                );
            }
        ).as_bytes())?;
    }}
    // Wait for the compiler to succeed.
    let status = child.wait()?;
    if status.success() {
        Ok(wasm_path)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("{:?} exited with status {}", cmd, status),
        ))
    }
}

pub(in crate)
fn compile (
    mod_name: &'_ Ident,
    input: TokenStream2,
) -> Result<TokenStream2>
{Ok({
    let debug = env::var("DEBUG_INLINE_MACROS").ok().map_or(false, |s| s == "1");
    if debug {
        println!("<<<\ncompile! {{");
        crate::utils::log_stream(input.to_string());
        println!("}}\n=== yields ===");
    }
    let mut file = ::syn::parse2(input)?;
    let macro_names_and_attrs = extract_macro_names_and_attrs(&mut file)?;
    let ref src = quote!( #file ).to_string();
    let ref wasm_path =
        compile_to_wasm(src)
            .map_err(|err| {
                if debug {
                    eprintln!("{}", err);
                }
                ::syn::Error::new(Span::call_site(),
                    "Compilation of the procedural macro failed",
                )
            })?
    ;
    let ret = macro_defs(mod_name, wasm_path, macro_names_and_attrs);
    if debug {
        crate::utils::log_stream(ret.to_string());
        println!(">>>\n");
    }
    ret
})}

fn macro_defs (
    mod_name: &'_ Ident,
    wasm_path: &'_ str,
    macro_names_and_attrs: Vec<(Ident, Vec<Attribute>)>,
) -> TokenStream2
{
    let mut ret = TokenStream2::new();
    macro_names_and_attrs.into_iter().for_each(|(name, attrs)| {
        ret.extend(quote_spanned! { name.span()=>
            #(#attrs)*
            macro_rules! #name {(
                $($proc_macro_input:tt)*
            ) => (
                // Defined in `eval.rs`
                $crate::#mod_name::__inline_proc_macros__eval_wasm__! {
                    #name
                    #wasm_path
                    $($proc_macro_input)*
                }
            )}
        });
    });
    ret.into()
}

fn extract_macro_names_and_attrs (file: &'_ mut ::syn::File)
  -> Result<Vec<(Ident, Vec<Attribute>)>>
{Ok({
    let mut macro_names_and_attrs = Vec::with_capacity(file.items.len());
    file.items.iter_mut().try_for_each(|item| Ok(match item {
        | &mut Item::Fn(ref mut func) => {
            if {
                // Check for the `proc_macro` attribute, and remove it.
                let mut skip = true;
                func.attrs.retain(|attr| if attr.path.is_ident("proc_macro") {
                    skip = false; // proc_macro fn requires further processing
                    false // pop attr
                } else {
                    true // keep attr
                });
                skip
            }
            {
                return Ok(());
            }
            let ref f_name = func.sig.ident;
            if matches!(func.vis, Visibility::Public(_)).not() {
                return Err(Error::new(
                    f_name.span(),
                    "`#[proc_macro]` function must be `pub`",
                ));
            }
            if let Some(ref abi) = func.sig.abi {
                return Err(Error::new(
                    abi.span(),
                    "`#[proc_macro]` function cannot have an `extern` annotation",
                ));
            }
            macro_names_and_attrs.push((
                f_name.to_owned(),
                ::core::mem::take(&mut func.attrs),
            ));
            // Transform the method into a wasm export.
            func.attrs.push(parse_quote!(#[no_mangle]));
            func.vis = parse_quote!(pub);
            func.sig.abi.replace(parse_quote!(extern "C"));
            func.block.stmts.insert(0, parse_quote! {
                ::proc_macro2::set_wasm_panic_hook();
            });
        },
        | _ => {},
    }))?;
    macro_names_and_attrs
})}

fn rustup_ensure_has_target (target: &'_ str)
  -> Result<(), Box<dyn ::std::error::Error>>
{Ok({
    let rustup = env::var("RUSTUP").ok();
    let rustup = rustup.as_deref().unwrap_or("rustup");
    let rustup = |args| {
        let mut cmd = Command::new(rustup);
        cmd.args(args);
        cmd
    };
    if  String::from_utf8(
            rustup(&["target", "list", "--installed"])
                .output()?
                .stdout
        )?
        .lines()
        .any(|s| s.trim() == target)
    {
        return Ok(())
    }
    let mut add_wasm_cmd = rustup(&["target", "add", target]);
    if let Err(err) = add_wasm_cmd.status() {
        eprintln!("Warning: command {:?} failed: {}", add_wasm_cmd, err);
    }
})}
