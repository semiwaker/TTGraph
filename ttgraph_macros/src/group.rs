use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{self, braced, Ident, Token};

use super::*;

pub(crate) struct NamedGroup {
  pub name: Ident,
  pub idents: Vec<Ident>,
}

impl Parse for NamedGroup {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let name: Ident = input.parse()?;
    let content;
    let _ = braced!(content in input);
    let inside = content.parse_terminated(Ident::parse, Token![,])?;
    Ok(NamedGroup {
      name,
      idents: inside.into_iter().collect(),
    })
  }
}

pub(crate) struct NamedGroupVec {
  pub groups: Vec<NamedGroup>,
}

impl Parse for NamedGroupVec {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let inside = input.parse_terminated(NamedGroup::parse, Token![,])?;
    Ok(NamedGroupVec { groups: inside.into_iter().collect() })
  }
}

pub(crate) fn make_get_links_by_group(
  links: &[LinkType], groups: &[Vec<Ident>],
) -> TokenStream {
  let mut group_map: BTreeMap<Ident, Vec<LinkType>> = BTreeMap::new();
  for (link, group) in links.iter().zip(groups.iter()) {
    for g in group {
      group_map.entry(g.clone()).or_default().push(link.clone());
    }
  }

  let mut arms = Vec::new();
  for (group_name, links) in group_map {
    let mut extends = Vec::new();
    for link in links {
      extends.push(match link {
        LinkType::Direct(ident, _) => {
          quote! {if !self.#ident.is_empty() {result.push(self.#ident);}}
        },
        LinkType::HSet(ident, _) => quote! {result.extend(self.#ident.clone());},
        LinkType::BSet(ident, _) => quote! {result.extend(self.#ident.clone());},
        LinkType::Vec(ident, _) => quote! {result.extend(self.#ident.clone());},
        LinkType::Empty => quote! {},
      });
    }
    arms.push(quote! {
      std::stringify!(#group_name) => {
        let mut result = Vec::new();
        #(#extends)*
        result
      },
    });
  }
  quote! {
    fn get_links_by_group(&self, name: &'static str) -> Vec<ttgraph::NodeIndex> {
      match name{
        #(#arms)*
        _ => vec![],
      }
    }
  }
}

pub(crate) fn make_in_group(groups: &Vec<NamedGroup>) -> TokenStream {
  let mut name_arms = Vec::new();
  for NamedGroup { name, idents } in groups {
    let mut matched_vars = Vec::new();
    for i in idents {
      matched_vars.push(quote! {Self::#i(_)});
    }
    name_arms.push(quote! {
      std::stringify!(#name) => match self {
        #(#matched_vars)|* => true,
        _ => false,
      },
    });
  }
  quote! {
    fn in_group(&self, name: &'static str) -> bool {
      match name {
        #(#name_arms)*
        _ => false,
      }
    }
  }
}
