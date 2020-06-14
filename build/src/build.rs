use ::std::{
    borrow::Cow,
    ops::{Not as _},
};

use ::anyhow::{anyhow, bail};
use ::cargo_metadata::{Metadata, MetadataCommand};

// Public dependency!
pub use ::anyhow::{
    Result,
};

define_strings! {
    const DEFAULT_IN_PATH = "src/proc_macros.rs";
    const DEFAULT_OUT_PATH = "$OUT_DIR/proc_macros.rs";
    const KEY = "inline_proc_macros";
}

/// Convenience macro to wrap the call to `run()` into a `fn main()`.
#[macro_export]
macro_rules! run {() => (
    fn main ()
      -> $crate::build::Result<()>
    {
        $crate::run()
    }
)}

pub
fn run ()
  -> Result<()>
{Ok({
    let metadata =
        MetadataCommand::new()
            .no_deps()
            .exec()?
    ;
    let in_out = metadata.fetch_in_out()?;
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed={}", in_out.0);
    let out = &*match interpolate_env_vars(in_out.1) {
        | Ok(it) => it,
        | Err(err) => return Err(anyhow!(
            concat!("when parsing `[package.metadata.", KEY!(), ".out] = {:?}`: {}"),
            in_out.1,
            err,
        )),
    };
    if already_up_to_date(in_out.0, out) { return Ok(()); }
    let ref input = ::std::fs::read_to_string(in_out.0)?;
    let input_ts = if let Ok(it) = input.parse() { it } else {
        // If we are here it means the input file could not even be properly
        // tokenized. At that point, we stop our generation and instead just
        // bundle the input as is, so as to let the compilation process proceed
        // as usual to lead a slightly better spanned error message.
        // We `#[cfg(...)]`-gate it out to prevent any semantic-related error
        // messages (may not be needed, but better be safe).
        ::std::fs::write(out, &format!(
            "#[cfg(any())]const _:()= {{\n\n\n{}\n\n\n}};",
            input,
        ))?;
        return Ok(());
    };
    let generated: String =
        match crate::compile_proc_macro::compile(input_ts) {
            | Ok(it) => it.to_string(),
            | Err(err) => {
                bail!("{}", crate::error::pp_syn_error(in_out.0, input, err));
            },
        }
    ;
    ::std::fs::write(out, generated.as_bytes())?;
})}

trait FetchInOut {
    fn fetch_in_out (self: &'_ Self)
      -> Result<(&'_ str, &'_ str)>
    ;
} impl FetchInOut for Metadata {
    fn fetch_in_out (self: &'_ Metadata)
    -> Result<(&'_ str, &'_ str)>
    {Ok({
        match self
                .packages
                .iter()
                .find({
                    let pkg_name = ::std::env::var("CARGO_PKG_NAME").unwrap();
                    move |package| package.name == pkg_name
                })
                .ok_or_else(|| anyhow!("Failed to find the current crate"))?
                .metadata
                .pointer(concat!("/", KEY!()))
        {
            | None => {
                println!(
                    concat!(
                        "cargo:warning=",
                        "`[package.metadata.", KEY!(), "]`",
                        " not found in the manifest file.",
                        " Defaulting paths to ",
                        "`in = {:?}`, ",
                        "`out = {:?}`, ",
                    ),
                    DEFAULT_IN_PATH,
                    DEFAULT_OUT_PATH,
                );
                (
                    DEFAULT_IN_PATH,
                    DEFAULT_OUT_PATH,
                )
            },
            | Some(json) => (
                json.pointer("/in")
                    .and_then(|it| it.as_str())
                    .ok_or_else(|| anyhow!(
                        "Invalid or missing key in \
                        `[package.metadata.{}]`, \
                        expected `in = \"some/path.rs\"`",
                        KEY,
                    ))?
                ,
                json.pointer("/out")
                    .and_then(|it| it.as_str())
                    .ok_or_else(|| anyhow!(
                        "Invalid or missing key in \
                        `[package.metadata.{}]`, \
                        expected `out = \"some/path.rs\"`",
                        KEY,
                    ))?
            ),
        }
    })}
}

fn interpolate_env_vars (input: &'_ str)
  -> Result<Cow<'_, str>>
{Ok({
    if input.contains('$').not() {
        return Ok(input.into());
    }
    let mut ret = String::with_capacity(256);
    let ref mut iter = input.char_indices().peekable();
    while let Some((_, c)) = iter.next() {
        match c {
            | '$' => {
                let var_name = match iter.next() {
                    | None => bail!("Invalid terminating `$`"),
                    | Some((_, '$')) => { // `$$` is an escaped `$`
                        ret.push('$');
                        continue;
                    },
                    | Some((open_pos, '{')) => {
                        let mut start = None;
                        let (end, _) =
                            iter.by_ref()
                                .inspect(|&(i, _)| {
                                    start.get_or_insert(i);
                                })
                                .find(|&(_, c)| c == '}')
                                .ok_or_else(|| anyhow!(
                                    "Unmatched opening `{{` at index {}", open_pos,
                                ))?
                        ;
                        &input[start.unwrap() .. end]
                    },
                    | Some((start, _)) => loop {
                        match iter.peek() {
                            | None => {
                                break &input[start ..];
                            },
                            | Some(&(end, '/'))
                            | Some(&(end, '\\'))
                            => {
                                break &input[start .. end];
                            },
                            | Some(_) => {
                                drop(iter.by_ref().next());
                            },
                        }
                    },
                };
                ret.push_str(&*
                    ::std::env::var(var_name)
                        .map_err(|err| anyhow!("{}: `{}`", err, var_name))?
                );
            },
            | _otherwise => {
                ret.push(c);
            },
        }
    }
    ret.into()
})}

/// Main point of using a build script with access to the filesystem.
/// 
/// We can use the generated file to "cache" the result of compiling the
/// procedural macro code to avoid rerunning it when, for instance, only the
/// `Cargo.toml` file changed.
fn already_up_to_date (
    in_: &'_ str,
    out: &'_ str,
) -> bool
{
    (|| Ok::<_, ::std::io::Error>(dbg!(
        ::std::path::Path::metadata(in_.as_ref())?
            .modified()?
        <
        ::std::path::Path::metadata(out.as_ref())?
            .modified()?
    )))().unwrap_or(false)
}
