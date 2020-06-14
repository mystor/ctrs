#[macro_use]
mod proc_macros {
    include! {
        concat!(env!("OUT_DIR"), "/proc_macros.rs")
    }
}

success! { 42 }
fn main ()
{}
