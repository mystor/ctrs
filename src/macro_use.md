Annotate a `mod` declaration to compile its `proc_macro` source code
into inlined `macro_rules!` definitions.

# Usage

  - **`build.rs`**

    ```rust
    fn main ()
    {
        println!("cargo:rustc-env=OUT_DIR={}", ::std::env::var("OUT_DIR").unwrap());
        println!("cargo:rustc-env=RUSTC={}", ::std::env::var("RUSTC").unwrap());
        println!("cargo:rerun-if-changed=build.rs");
    }
    ```

    <details><summary>Remarks</summary>

    This indeed hints at the procedural macros doing unorthodox stuff, such
    as invoking `rustc` or using the `OUT_DIR` filesystem, which are both things
    that a `build.rs` script, instead of a procedural macro, ought to do.

    While it is possible to implement the logic of this attribute macro with
    a `build.rs` script, the ergonomics would suffer a lot, making its usability
    hardly better that that of using an external `proc_macro = "true"` crate,
    defeating the purpose of the crate.

    That being said, Rust does not guarantee that procedural macros be allowed
    to interact with their environment like they currently do (it's rather an
    unfortunate byproduct of how they are implemented), so **be aware that this
    non `build.rs`-based approach could break with a new Rust release**.

    That's why it isn't super advisable to use this crate in production.

    That being said, by the time Rust does that change, if it ever does,
    we should have a full-featured WASM-encapsulation mechanism for procedural
    macros, which should incidentally allow to trivially implement what this
    crate achieves, that is, having inline procedural macros built-in into the
    build ecosystem, rendering this very crate obsolete.

    In other words, by the time this crate breaks, we won't be needing it 🙂

    </summary>

      - Using a `{}` instead of a `;` is necessary since in stable Rust
        a procedural macro attribute cannot be applied to a `mod name;`
        declaration.

      - Also, only the top-level module (`src/{lib,main}.rs`) is supported,
        due to limitations of procedural macros (they cannot know whence they
        are called), so the source code will always be loaded from
        `src/some_module_name.rs`.
    
    </details>

  - **`src/lib.rs`** (or `src/main.rs`)

    ```rust,no_run
    #[inline_proc_macros::macro_use]
    mod some_module_name {}

    // ...
    ```

    <details><summary>Remarks</summary>

      - Using a `{}` instead of a `;` is necessary since in stable Rust
        a procedural macro attribute cannot be applied to a `mod name;`
        declaration.

      - Also, only the top-level module (`src/{lib,main}.rs`) is supported,
        due to limitations of procedural macros (they cannot know whence they
        are called), so the source code will always be loaded from
        `src/some_module_name.rs`.
    
    </details>

  - **`src/some_module_name.rs`**

    ```rust,ignore
    use ::proc_macro2::TokenStream;

    /// Some docstring    (optional)
    #[macro_export]     // optional
    #[proc_macro]       // required
    pub                 // required
    fn some_macro_name (input: TokenStream)
      -> TokenStream
    {
        // ...
    }
    ```

      - `#[inline_proc_macros::macro_use]` will automagically compile this code
        and convert it into something along the lines of:

        ```rust,no_run
        /// Some docstring
        #[macro_export] // if present in the above code
        macro_rules! some_macro_name { /* compiled procedural macro */ }
        ```

## External crates

The only external crates available from within the `proc_macro` code are:

  - [`::quote`](https://docs.rs/quote)

  - [`::syn`](https://docs.rs/syn)

## Examples

### Reversing the `char`s of a string literal at compile-time

```rust,ignore
//! src/proc_macros.rs

use ::proc_macro2::TokenStream;
use ::syn::*;

#[proc_macro] pub
fn reverse (input: TokenStream)
  -> TokenStream
{
    let input: LitStr = parse_macro_input!(input);
    let mut string: String = input.value();
    string = string.chars().rev().collect();
    ::quote::quote!(
        #string
    ).into()
}
```

```rust,ignore
//! src/main.rs

#[inline_proc_macros::macro_use]
mod proc_macros {}

fn main ()
{
    assert_eq!(
        reverse!("!dlroW ,olleH"),
        "Hello, World!",
    )
}
```

### FFI-compatible string literals

No more:

  - `CStr::from_bytes_with_nul(b"Hello, World!\0").unwrap()`,

  - or worse, `CString::new("Hello, World").unwrap()`!

Instead, you can do:

```rust,ignore
//! src/proc_macros.rs

use ::proc_macro2::TokenStream;
use ::syn::*;

/// Converts an input string literal into a `&'static CStr`, after appending
/// a terminating null-byte if none were found, and producing a **compile-time
/// error** (instead of at runtime!) if the string literal contains an inner
/// null byte.
#[proc_macro] pub
fn c_str (input: TokenStream)
  -> TokenStream
{
    let input: LitStr = parse_macro_input!(input);
    let ref mut bytes: Vec<u8> = input.value().into();
    match bytes.iter().position(|&&b| b == b'\0') {
        | Some(idx) if idx + 1 == string.len() => {
            // the string literal already has a terminating null byte
        },
        | None => {
            // No null byte at all
            bytes.push(b'\0');
        },
        | Some(bad_idx) => {
            // Inner null byte!
            return Error::new_spanned(
                input,
                &format!("Error, found inner null byte at index {}", bad_idx),
            ).to_compile_error().into();
        },
    }
    let checked_byte_str = LitByteStr::new(
        bytes,
        input.span(),
    );
    // Now `bytes` is null-terminated and has no inner nulls.
    ::quote::quote!(
        unsafe {
            ::std::ffi::CStr::from_bytes_with_nul_unchecked(
                #checked_byte_str
            )
        }
    ).into()
}
```

  - <details><summary><code>no_std</code>-compatible expansion</summary>

    It's just a matter of expanding to a `&'static [u8]` instead of
    `&'static CStr`:

    ```rust,no_run
    // Now `bytes` is null-terminated and has no inner nulls.
    ::quote::quote!(
        {
            // Use a constant to coerce to a slice while remaining
            // `const`-compatible
            const IT: &'static [u8] = #checked_byte_str;
            IT
        }
    ).into()
    ```
    </details>

```rust,ignore
//! src/main.rs

use ::std::ffi::CStr;

#[inline_proc_macros::macro_use]
mod proc_macros {}

fn main ()
{
    let hello: &'static CStr = c_str!("Hello, World");
    // let err: &'static CStr = c_str!("Hell\0, World!"); /* Compile error */
    unsafe {
        ::libc::puts(hello.as_ptr());
    }
}
```
