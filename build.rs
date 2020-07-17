#[cfg(not(feature = "docs"))]
fn main ()
  -> Result<(), Box<dyn ::std::error::Error>>
{Ok({
    use ::std::{
        env,
        ops::Not as _,
        process::Command,
    };

    /// Helper function to auto-install the required target (_e.g._,
    /// `wasm32-unknown-unknown`) if it is not the case.
    /// 
    /// It's a bit of a convenience hack, this is not expected to always work, at
    /// all, but it should on most classic setups, and would allow in that case for
    /// people to "forget" to install that target.
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
        // By this time, there _is_ a `rustup` binary on the host, which does not
        // have the target in it. So now we can try to install it, and no longer
        // swallow / hide the errors that may occur, as they now be meaningful.
        let mut add_wasm_cmd = rustup(&["target", "add", target]);
        if let Err(err) = add_wasm_cmd.status() {
            println!("cargo:warning=command {:?} failed: {}", add_wasm_cmd, err);
        }
    })}

    let _ = rustup_ensure_has_target("wasm32-unknown-unknown");
    let ref manifest_dir = env::var("CARGO_MANIFEST_DIR")?;

    // NOTE: The `.wasm` file would be sound to bundle, however the format of
    // `.rlib` files is unstable. Given that the rlibs for proc_macro2, syn, and
    // quote need to be built anyway, build everything.
    let ref target_dir = env::var("OUT_DIR")?;
    let mut cargo_cmd = Command::new(env::var("CARGO")?);
    cargo_cmd.args(&[
        "build", "-vv", "--release", "--offline",
        "--manifest-path", &[manifest_dir, "/wasm/Cargo.toml"].concat(),
        "-p", "placeholder",
        "-p", "proc-macro2",
        "-p", "syn",
        "-p", "quote",
        "-p", "unicode-xid",
        "--target", "wasm32-unknown-unknown",
        "--target-dir", target_dir,
        "--features",
            if cfg!(feature = "trace-macros") {
                "extra-traits"
            } else {
                ""
            }
        ,
    ]);
    // Run the command
    let status = cargo_cmd.status()?;
    if status.success().not() {
        panic!("{:?} exited with status {}", cargo_cmd, status);
    }

    // Set the `RUSTC` env var for use by the proc-macro.
    println!("cargo:rustc-env=RUSTC={}", env::var("RUSTC")?);
    // Re-run this script only if the `wasm` module is changed
    [
        "wasm/Cargo.toml",
        "wasm/Cargo.lock",
        "wasm/src/lib.rs",
        "wasm/.cargo/config",
    ].iter().for_each(|&path| {
        println!("cargo:rerun-if-changed={}/{}", manifest_dir, path);
    });
})}

#[cfg(feature = "docs")]
fn main ()
{}
