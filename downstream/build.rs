fn main ()
{
    println!("cargo:rustc-env=OUT_DIR={}", ::std::env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-env=RUSTC={}", ::std::env::var("RUSTC").unwrap());
    println!("cargo:rerun-if-changed=build.rs");
}
