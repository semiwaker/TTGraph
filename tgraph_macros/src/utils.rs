use change_case::pascal_case;
use quote::format_ident;
use syn::Ident;

pub(crate) fn upper_camel(ident: &Ident) -> Ident {
  format_ident!("{}", pascal_case(&ident.to_string()))
}
