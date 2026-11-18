use crate::expansion::naming::normalize;
use crate::expansion::structure::require_plain_struct;
use quote::quote;
use syn::{DeriveInput, LitStr};

pub(crate) fn expand_sub_config(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    require_plain_struct(input)?;
    let mut prefix = None;
    let mut section = None;
    let mut runtime: syn::Path = syn::parse_quote!(::xt_config);
    for attr in &input.attrs {
        if attr.path().is_ident("env") {
            attr.parse_nested_meta(|meta| {
                let value: LitStr = meta.value()?.parse()?;
                if meta.path.is_ident("prefix") {
                    prefix = Some(value);
                } else if meta.path.is_ident("section") {
                    section = Some(value);
                } else if meta.path.is_ident("crate_path") {
                    runtime = value.parse()?;
                } else if !meta.path.is_ident("validate") {
                    return Err(meta.error("Supported options: prefix, section, validate, crate_path"));
                }
                Ok(())
            })?;
        }
    }
    let prefix =
        prefix.ok_or_else(|| syn::Error::new_spanned(input, "SubConfig is missing env prefix"))?;
    let section =
        section.ok_or_else(|| syn::Error::new_spanned(input, "SubConfig is missing env section"))?;
    let prefix = normalize(&prefix.value(), prefix.span())?;
    let section = section
        .value()
        .split('.')
        .map(|segment| normalize(segment, section.span()))
        .collect::<syn::Result<Vec<_>>>()?
        .join("__");
    let name = &input.ident;
    Ok(quote! {
        #runtime::__inventory::submit! {
            #runtime::registry::Registration {
                prefix: #prefix,
                section: #section,
                name: ::std::concat!(::std::module_path!(), "::", ::std::stringify!(#name)),
                load: || ::std::result::Result::Ok(::std::boxed::Box::new(#name::new()?)),
            }
        }
    })
}
