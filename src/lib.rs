extern crate proc_macro;

use proc_macro::{TokenStream, TokenTree};
use std::fs;
use std::io::{self, Write};
use std::iter;
use std::process::{Command, Stdio};
use std::env;
use tempdir::TempDir;

// Crates provided as part of the runtime
const UNICODE_XID_RLIB: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/wasm32-unknown-unknown/release/libunicode_xid.rlib"
));
const PROC_MACRO2_RLIB: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/wasm32-unknown-unknown/release/libproc_macro2.rlib"
));
const SYN_RLIB: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/wasm32-unknown-unknown/release/libsyn.rlib"
));
const QUOTE_RLIB: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/wasm32-unknown-unknown/release/libquote.rlib"
));

// WAsM module with core impl logic
const IMPL_WA: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/wasm32-unknown-unknown/release/ctrs_wasm.wasm"
));

/// Invoke rustc to build a `wasm32-unknown-unknown` crate with dependencies on
/// `unicode_xid`, `proc_macro2`, `syn`, and `quote`.
fn build_code(name: &str, source: &str) -> io::Result<Vec<u8>> {
    // Build within a tempdir
    let tmp = TempDir::new("ctrs_build")?;
    let wasm_path = tmp.path().join(format!("{}.wasm", name));

    // Write out the rlibs
    let unicode_xid_path = tmp.path().join("libunicode_xid.rlib");
    let proc_macro2_path = tmp.path().join("libproc_macro2.rlib");
    let syn_path = tmp.path().join("libsyn.rlib");
    let quote_path = tmp.path().join("libquote.rlib");
    fs::write(&unicode_xid_path, UNICODE_XID_RLIB)?;
    fs::write(&proc_macro2_path, PROC_MACRO2_RLIB)?;
    fs::write(&syn_path, SYN_RLIB)?;
    fs::write(&quote_path, QUOTE_RLIB)?;

    // Run the compiler
    let mut child = Command::new(env!("RUSTC"))
        .stdin(Stdio::piped())
        .args(&[
            "--target",
            "wasm32-unknown-unknown",
            "--edition",
            "2018",
            "--crate-type",
            "cdylib",
            "--crate-name",
            name,
            "-o",
            wasm_path.to_str().unwrap(),
            "-L",
            &format!("dependency={}", tmp.path().to_str().unwrap()),
            "--extern",
            &format!("unicode_xid={}", unicode_xid_path.to_str().unwrap()),
            "--extern",
            &format!("proc_macro2={}", proc_macro2_path.to_str().unwrap()),
            "--extern",
            &format!("syn={}", syn_path.to_str().unwrap()),
            "--extern",
            &format!("quote={}", quote_path.to_str().unwrap()),
            "-",
        ])
        .spawn()?;
    child.stdin.take().unwrap().write_all(source.as_bytes())?;

    let status = child.wait()?;
    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("rustc exited with status {}", status),
        ));
    }

    // Read in the resulting wasm file
    Ok(fs::read(&wasm_path)?)
}

fn log_stream(ts: &TokenStream) -> String {
    let in_str = ts.to_string();
    if in_str.len() > 1000 {
        let pre = in_str.chars().take(400).collect::<String>();
        let post = in_str.chars().rev().take(400).collect::<String>().chars().rev().collect::<String>();
        format!("{} [.. {} chars ..] {}", pre, in_str.len() - 800, post)
    } else {
        in_str
    }
}

#[proc_macro]
pub fn ctrs(ts: TokenStream) -> TokenStream {
    let ctrs_log = !env::var("CTRS_LOG").unwrap_or_default().is_empty();
    if ctrs_log {
        println!("ctrs!({})", log_stream(&ts));
    }

    // Check if we're dealing with an internal message.
    let mut iter = ts.clone().into_iter();
    let first_id = iter.next().map(|id| id.to_string()).unwrap_or_default();
    let os = match &first_id[..] {
        // ctrs!(__build_wasm__ $krate {..src..} ...) => build_result($krate "b64" ...)
        "__build_wasm__" => {
            let krate = iter.next().expect("missing crate name");
            let source = match iter.next().expect("missing crate body") {
                TokenTree::Group(grp) => grp.stream().to_string(),
                _ => panic!("expected crate body block"),
            };

            let wasm = build_code(&krate.to_string(), &source).expect("error building crate");
            let wasm_lit = format!("\"{}\"", base64::encode(&wasm))
                .parse::<TokenStream>()
                .unwrap();

            // $krate "b64str" ...
            let mut mac_args = TokenStream::new();
            mac_args.extend(iter::once(krate));
            mac_args.extend(wasm_lit);
            mac_args.extend(iter);

            watt::proc_macro("build_result", mac_args, IMPL_WA)
        }

        // ctrs!(__eval_wasm__ $mname "b64str" ...) => result of calling method
        "__eval_wasm__" => {
            let func = iter.next().expect("missing func name").to_string();
            let wasm_lit = iter.next().expect("missing wasm src").to_string();

            assert!(wasm_lit.starts_with('"') && wasm_lit.ends_with('"'));
            let wasm = base64::decode(&wasm_lit[1..wasm_lit.len() - 2]).unwrap();

            watt::proc_macro(&func, iter.collect(), &wasm)
        }

        // Not an internal method! Hand over.
        _ => watt::proc_macro("ctrs", ts, IMPL_WA),
    };

    if ctrs_log {
        println!("  => {}", log_stream(&os));
    }
    os
}
