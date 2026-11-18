use crate::expansion::structure::require_plain_struct;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr};

pub(crate) fn expand_config(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    require_plain_struct(input)?;
    if !matches!(&input.data, Data::Struct(data) if matches!(&data.fields, Fields::Named(fields) if fields.named.is_empty()))
    {
        return Err(syn::Error::new_spanned(
            input,
            "Config requires an empty named struct, such as struct AppConfig {}",
        ));
    }
    let mut runtime: syn::Path = syn::parse_quote!(::xt_config);
    for attr in &input.attrs {
        if attr.path().is_ident("config") {
            attr.parse_nested_meta(|meta| {
                if !meta.path.is_ident("crate_path") {
                    return Err(meta.error("Supported option: crate_path"));
                }
                runtime = meta.value()?.parse::<LitStr>()?.parse()?;
                Ok(())
            })?;
        }
    }
    let name = &input.ident;
    Ok(quote! {
        impl #name {
            pub fn new() -> #runtime::Result<Self> {
                #runtime::init_registered()?;
                ::std::result::Result::Ok(Self {})
            }

            pub fn get<T: ::std::marker::Send + ::std::marker::Sync + 'static>(
                &self,
            ) -> #runtime::Result<&'static T> {
                #runtime::get::<T>()
            }
        }
    })
}
