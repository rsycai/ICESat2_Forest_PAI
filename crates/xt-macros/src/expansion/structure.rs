use syn::{Data, DeriveInput};

pub(super) fn require_plain_struct(input: &DeriveInput) -> syn::Result<()> {
    if !input.generics.params.is_empty() || !matches!(input.data, Data::Struct(_)) {
        return Err(syn::Error::new_spanned(input, "Configuration derive supports nongeneric structs only"));
    }
    Ok(())
}
