use crate::expansion::naming::normalize;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr};

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let mut prefix = None;
    let mut section = None;
    let mut validate = None;
    let mut runtime: syn::Path = syn::parse_quote!(::xt_config);
    for attr in &input.attrs {
        if !attr.path().is_ident("env") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            let value: LitStr = meta.value()?.parse()?;
            if meta.path.is_ident("prefix") {
                prefix = Some(value);
            } else if meta.path.is_ident("section") {
                section = Some(value);
            } else if meta.path.is_ident("validate") {
                validate = Some(value.parse::<syn::Ident>()?);
            } else if meta.path.is_ident("crate_path") {
                runtime = value.parse()?;
            } else {
                return Err(meta.error("Supported options: prefix, section, validate, crate_path"));
            }
            Ok(())
        })?;
    }
    let prefix = prefix.ok_or_else(|| {
        syn::Error::new_spanned(input, "Missing #[env(prefix = \"XT\", section = \"logging\")]")
    })?;
    let section = section.ok_or_else(|| syn::Error::new_spanned(input, "Missing section"))?;
    let mut base = normalize(&prefix.value(), prefix.span())?;
    for segment in section.value().split('.') {
        base.push_str("__");
        base.push_str(&normalize(segment, section.span())?);
    }
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "EnvLoad does not support generic configuration structs",
        ));
    }
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "EnvLoad requires a struct with named fields",
                ));
            }
        },
        _ => return Err(syn::Error::new_spanned(input, "EnvLoad supports structs only")),
    };
    let mut setters = Vec::new();
    let mut defaults = Vec::new();
    let mut generate_default = false;
    let mut mapped_keys = std::collections::BTreeSet::new();
    for field in fields {
        let name = field.ident.as_ref().expect("Named fields");
        let mut key = name.to_string().trim_start_matches("r#").to_owned();
        let mut aliases = Vec::<LitStr>::new();
        let mut os = false;
        let mut json = false;
        let mut skip = false;
        let mut default = None::<syn::Expr>;
        for attr in &field.attrs {
            if !attr.path().is_ident("env") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("os") {
                    os = true;
                } else if meta.path.is_ident("json") {
                    json = true;
                } else if meta.path.is_ident("skip") {
                    skip = true;
                } else if meta.path.is_ident("key") {
                    key = meta.value()?.parse::<LitStr>()?.value();
                } else if meta.path.is_ident("alias") {
                    aliases.push(meta.value()?.parse()?);
                } else if meta.path.is_ident("default") {
                    if default.is_some() {
                        return Err(meta.error("default must not be declared more than once"));
                    }
                    default = Some(meta.value()?.parse()?);
                } else {
                    return Err(meta.error("Supported options: key, alias, os, json, skip, default"));
                }
                Ok(())
            })?;
        }
        if os && json {
            return Err(syn::Error::new_spanned(field, "os and json cannot be used together"));
        }
        let default = match default {
            Some(expression) => {
                generate_default = true;
                if matches!(&expression, syn::Expr::Lit(literal) if matches!(literal.lit, syn::Lit::Str(_)))
                {
                    quote!(::std::convert::Into::into(#expression))
                } else {
                    quote!(#expression)
                }
            }
            None => quote!(::std::default::Default::default()),
        };
        defaults.push(quote!(#name: #default));
        if skip {
            continue;
        }
        let key = format!("{base}__{}", normalize(&key, name.span())?);
        if !mapped_keys.insert(key.clone()) {
            return Err(syn::Error::new_spanned(
                field,
                "Configuration fields map to duplicate environment variable names",
            ));
        }
        let key = LitStr::new(&key, name.span());
        for alias in &aliases {
            if alias.value().is_empty() || alias.value().contains(['=', '\0']) {
                return Err(syn::Error::new_spanned(alias, "Invalid environment variable alias"));
            }
        }
        let keys = quote!(&[#key, #(#aliases),*]);
        let read = if os {
            quote!(#runtime::env_load::read_os(#keys)?.map(::std::convert::Into::into))
        } else if json {
            quote!(#runtime::env_load::read_json(#keys)?)
        } else {
            quote!(#runtime::env_load::read(#keys)?)
        };
        setters.push(quote! {
            if let ::std::option::Option::Some(value) = #read {
                __config.#name = value;
            }
        });
    }
    let validation = validate.map(|method| quote!(let _ = __config.#method()?;));
    let name = &input.ident;
    let default_impl = generate_default.then(|| {
        quote! {
            impl ::std::default::Default for #name {
                fn default() -> Self {
                    Self { #(#defaults),* }
                }
            }
        }
    });
    Ok(quote! {
        #default_impl
        impl #name {
            pub fn new() -> #runtime::Result<Self> {
                #runtime::ensure_loaded()?;
                #runtime::env_load::with_file_defaults(Self::load_from_env)
            }

            pub fn load_from_env() -> #runtime::Result<Self> {
                let mut __config = <Self as ::std::default::Default>::default();
                #(#setters)*
                #validation
                ::std::result::Result::Ok(__config)
            }
        }
    })
}
