use ::proc_macro::TokenStream;

pub(in crate)
fn eval_wasm (input: TokenStream)
  -> TokenStream
{
    #[cfg(feature = "trace-macros")] {
        println!("\n__eval_wasm__! {{");
        crate::utils::log_stream(input.to_string());
        println!("}}\n");
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
    let compiled_wasm_path = &wasm_lit[1 .. wasm_lit.len() - 1];
    let ref wasm =
        ::std::fs::read(debug!(compiled_wasm_path))
            .unwrap_or_else(|err| panic!(
                "Failed to read the file `{}`: {}",
                compiled_wasm_path,
                err,
            ))
    ;
    let input: TokenStream = tokens.collect();
    #[cfg(feature = "trace-macros")] {
        println!("\n<<<\n{}! {{ {} }}\n=== yields ===\n", func, input.to_string());
    }
    let expanded = ::watt::WasmMacro::new(wasm).proc_macro(&func, input.into());
    #[cfg(feature = "trace-macros")] {
        println!("\n{}\n\n>>>", expanded.to_string());
    }
    expanded
}
