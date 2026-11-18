pub(super) fn normalize(name: &str, span: proc_macro2::Span) -> syn::Result<String> {
    if name.is_empty()
        || name.contains("__")
        || name.starts_with('_')
        || name.ends_with('_')
        || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(syn::Error::new(
            span,
            "Names allow ASCII letters, digits, and single interior underscores only",
        ));
    }
    Ok(name.to_ascii_uppercase())
}
