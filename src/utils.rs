macro_rules! define_strings {(
    $(
        const $VAR:ident = $($s:expr),* $(,)? ;
    )*
) => (
    $(
        #[allow(unused)]
        macro_rules! $VAR { () => (
            concat!($($s),*)
        )}
        #[allow(unused)]
        const $VAR: &str = $VAR!();
    )*
)}
