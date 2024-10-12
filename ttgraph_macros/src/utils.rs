use change_case::{pascal_case, self};
use quote::{format_ident, quote, ToTokens};
use syn::{Ident, Visibility};
use proc_macro2::TokenStream;

pub(crate) fn upper_camel(ident: &Ident) -> Ident {
  format_ident!("{}", pascal_case(&ident.to_string()), span = ident.span())
}

pub(crate) fn snake_case(ident: &Ident) -> Ident {
  format_ident!("{}", change_case::snake_case(&ident.to_string()), span = ident.span())
}

pub(crate) fn make_generated_mod(result: &mut TokenStream, generated: TokenStream, ident: &Ident, vis: &Visibility) -> Ident {
  let gen_ident = format_ident!("ttgraph_gen_{}", change_case::snake_case(&ident.to_string()), span=ident.span());
  quote!{
    #vis mod #gen_ident {
      #generated
    }
  }.to_tokens(result);
  gen_ident
}
