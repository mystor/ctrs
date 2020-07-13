use ::proc_macro::TokenStream;

pub(in crate)
fn eval_wasm (input: TokenStream)
  -> TokenStream
{
    let debug =
        ::std::env::var("DEBUG_INLINE_MACROS")
            .ok()
            .map_or(false, |s| s == "1")
    ;
    mk_debug!(if debug);
    if debug {
        println!("<<<__eval_wasm__! {{");
        crate::utils::log_stream(input.to_string());
        println!("}}\n>>>");
    }
    let mut tokens = TokenStream::into_iter(input.into());
    let func =
        tokens
            .next()
            .expect("Missing proc_macro name")
            .to_string()
    ;
    let wasm_lit =
        tokens
            .next()
            .expect("Missing WASM-compiled proc_macro source code")
            .to_string()
    ;
    assert!(wasm_lit.starts_with('"') && wasm_lit.ends_with('"'));
    let file_id = &wasm_lit[1 .. wasm_lit.len() - 1];
    let ref compiled_wasm_path = format!(
        COMPILED_WASM_PATH_TEMPLATE!(),
        out_dir = renv!("OUT_DIR"),
        file_id = debug!(file_id),
    );
    let ref wasm =
        ::std::fs::read(debug!(compiled_wasm_path))
            .unwrap_or_else(|err| panic!(
                "Failed to read the file `{}`: {}",
                compiled_wasm_path,
                err,
            ))
    ;
    let input: TokenStream = tokens.collect();
    if debug {
        println!("\n<<<\n{}! {{ {} }}\n=== yields ===\n", func, input.to_string());
    }
    let expanded = ::watt::proc_macro(&func, input.into(), wasm);
    if debug {
        println!("\n{}\n\n>>>", expanded.to_string());
    }
    expanded
}
