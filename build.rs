use std::env;
use std::error::Error;
use std::process::Command;

fn main() -> Result<(), Box<dyn Error>> {
    // Set the `RUSTC` env var for use by the proc-macro.
    let rustc = env::var("RUSTC")?;
    println!("cargo:rustc-env=RUSTC={}", rustc);

    // Re-run this script only if the `wasm` module is changed
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    println!(
        "\
cargo:rerun-if-changed={0}/wasm/Cargo.toml
cargo:rerun-if-changed={0}/wasm/Cargo.lock
cargo:rerun-if-changed={0}/wasm/src/lib.rs
cargo:rerun-if-changed={0}/wasm/.cargo/config",
        manifest_dir,
    );

    // NOTE: The `.wasm` file would be sound to bundle, however the format of
    // `.rlib` files is unstable. Given that the rlibs for proc_macro2, syn, and
    // quote need to be built anyway, build everything.
    let out_dir = env::var("OUT_DIR")?;
    let status = Command::new(env::var("CARGO")?)
        .args(&[
            "build",
            "-vv",
            "--target",
            "wasm32-unknown-unknown",
            "--target-dir",
            &out_dir,
            "--frozen",
            "--release",
            "--manifest-path",
            &format!("{}/wasm/Cargo.toml", manifest_dir),
            "-p",
            "ctrs-wasm",
            "-p",
            "proc-macro2",
            "-p",
            "syn",
            "-p",
            "quote",
            "-p",
            "unicode-xid",
        ])
        .status()?;
    if !status.success() {
        panic!("cargo exited with status {}", status);
    }

    Ok(())
}
