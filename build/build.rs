use ::std::{
    env,
    ops::Not as _,
    process::Command,
};

fn main ()
  -> Result<(), Box<dyn ::std::error::Error>>
{Ok({
    // Set the `RUSTC` env var for use by the proc-macro.
    println!("cargo:rustc-env=RUSTC={}", env::var("RUSTC")?);

    let ref manifest_dir = env::var("CARGO_MANIFEST_DIR")?;

    // Re-run this script only if the `wasm` module is changed
    [
        "wasm/Cargo.toml",
        "wasm/Cargo.lock",
        "wasm/src/lib.rs",
        "wasm/.cargo/config",
    ].iter().for_each(|&path| {
        println!("cargo:rerun-if-changed={}/{}", manifest_dir, path);
    });

    // NOTE: The `.wasm` file would be sound to bundle, however the format of
    // `.rlib` files is unstable. Given that the rlibs for proc_macro2, syn, and
    // quote need to be built anyway, build everything.
    let ref target_dir = env::var("OUT_DIR")?;
    let mut cmd = Command::new(env::var("CARGO")?);
    let status =
        cmd
        .args(&[
            "build", "-vv",
            "--manifest-path", &format!("{}/wasm/Cargo.toml", manifest_dir),
            "-p", "ctrs-wasm",
            "-p", "proc-macro2",
            "-p", "syn",
            "-p", "quote",
            "-p", "unicode-xid",
            "--target", "wasm32-unknown-unknown",
            "--release", "--frozen",
            "--target-dir", target_dir,
        ])
        .status()?
    ;
    if status.success().not() {
        panic!("{:?} exited with status {}", cmd, status);
    }
})}
