pub(in crate)
fn pp_syn_error (
    source_file: &'_ str,
    source_file_contents: &'_ str,
    err: ::syn::Error,
) -> String
{
    let span = err.span();
    let (start, end) = (span.start(), span.end());
    let line_size = format!("{}", end.line).len(); // FIXME: do this properly
    let context = if end.line <= start.line || true /* TODO */ {
        format!(
            concat!(
                " {line} | {code}\n",
                " {0:>line_size$} | {0:>start_col$}{0:^^span_len$}",
            ), "",
            line = start.line,
            code = source_file_contents.lines().nth(start.line - 1).unwrap(),
            line_size = line_size,
            start_col = start.column,
            span_len = ::core::cmp::max(1, end.column.saturating_sub(start.column)),
        )
    } else {
        todo!()
    };
    format!(
        concat!(
            "{err_msg}\n",
            " --> {source_file}:{line}:{col}\n",
            " {:>line_size$} |\n",
            "{context}\n",
        ), "",
        source_file = source_file,
        err_msg = err,
        line = start.line,
        col = start.column + 1,
        line_size = line_size,
        context = context,
    )
}
