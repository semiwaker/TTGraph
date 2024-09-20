use std::collections::{BTreeMap, BTreeSet, HashMap};

use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{self, custom_punctuation, parse2, Ident, Token, Type};

use crate::utils::*;
use crate::NamedGroup;

custom_punctuation!(BidirectionalSep, <->);

#[derive(Clone, Debug)]
pub(crate) struct BidirectionalLink {
  pub var1: Ident,
  pub link1: Ident,
  pub var2: Ident,
  pub link2: Ident,
}

impl Parse for BidirectionalLink {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    Ok({
      let var1: Ident = input.parse()?;
      let _: Token![.] = input.parse()?;
      let link1: Ident = input.parse()?;
      let _: BidirectionalSep = input.parse()?;
      let var2: Ident = input.parse()?;
      let _: Token![.] = input.parse()?;
      let link2: Ident = input.parse()?;
      BidirectionalLink { var1, link1, var2, link2 }
    })
  }
}

struct BidirectionalLinkVec {
  links: Vec<BidirectionalLink>,
}

impl Parse for BidirectionalLinkVec {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let punct = input.parse_terminated(BidirectionalLink::parse, Token![,])?;

    Ok(BidirectionalLinkVec { links: punct.into_iter().collect() })
  }
}

pub(crate) fn get_bidirectional_links(
  tokens: TokenStream, links: &mut Vec<BidirectionalLink>,
) -> syn::Result<()> {
  let link_vec: BidirectionalLinkVec = parse2(tokens)?;
  links.extend(link_vec.links);
  Ok(())
}

pub(crate) fn check_bidirectional_links(
  vars: &[(Ident, Type)], links: &[BidirectionalLink], groups: &[NamedGroup]
) {
  let mut vars = BTreeSet::from_iter(vars.iter().map(|(ident, _)| ident.clone()));
  vars.extend(groups.iter().map(|x|x.name.clone()));
  for l in links {
    if !vars.contains(&l.var1) {
      emit_error!(l.var1, "Unknown identifier, not a variant of the NodeEnum");
    }
    if !vars.contains(&l.var2) {
      emit_error!(l.var2, "Unknown identifier, not a variant of the NodeEnum");
    }
  }
}


pub(crate) fn expand_bidirectional_links(
  links: Vec<BidirectionalLink>, groups: &[NamedGroup]
) -> Vec<BidirectionalLink> {
  let mut result = Vec::new();
  let groups = BTreeMap::from_iter(groups.iter().map(|x|(x.name.clone(), x.idents.clone())));
  for l in links {
    let var1 = if let Some(g) = groups.get(&l.var1) {
      g.clone()
    } else {
      vec![l.var1]
    };
    let var2 = if let Some(g) = groups.get(&l.var2) {
      g.clone()
    } else {
      vec![l.var2]
    };

    for v1 in var1 {
      for v2 in &var2 {
        result.push(BidirectionalLink { var1: v1.clone(), link1: l.link1.clone(), var2: v2.clone(), link2:l.link2.clone() });
      }
    }
  }
  result
}

pub(crate) fn make_bidirectional_link(
  vars: &[(Ident, Type)], links: &[BidirectionalLink],
) -> TokenStream {
  let mut b_links: BTreeMap<Ident, BTreeMap<Ident, BTreeSet<(Ident, Ident)>>> =
    BTreeMap::new();
  let mut ty_map: HashMap<Ident, Type> = HashMap::new();

  for (var, ty) in vars {
    ty_map.insert(var.clone(), ty.clone());
  }

  for link in links {
    b_links
      .entry(link.var1.clone())
      .or_default()
      .entry(link.link1.clone())
      .or_default()
      .insert((link.var2.clone(), link.link2.clone()));
    b_links
      .entry(link.var2.clone())
      .or_default()
      .entry(link.link2.clone())
      .or_default()
      .insert((link.var1.clone(), link.link1.clone()));
  }

  let mut link_mirrors_of_arms = Vec::new();
  for (var, ty) in vars {
    if let Some(v) = b_links.get(var) {
      let mut arms = Vec::new();
      for (link, to) in v {
        let camel = upper_camel(link);
        let mut possible_links = Vec::new();
        for (var2, link2) in to {
          let var2_ty = &ty_map[var2];
          let link2_camel = upper_camel(link2);
          possible_links.push(quote!{
              Self::LoGMirrorEnum::#var2(<#var2_ty as ttgraph::TypedNode>::LoGMirror::#link2_camel)
            });
        }
        arms.push(quote! {
          <#ty as ttgraph::TypedNode>::LoGMirror::#camel => [#(#possible_links),*].into_iter().flat_map(|l|Self::expand_link_groups(l)).collect(),
        });
      }
      link_mirrors_of_arms.push(quote! {
        Self::LoGMirrorEnum::#var(l) => {
          match l {
            #(#arms)*
            _ => vec![],
          }
        }
      });
    }
  }

  let mut links_arms = Vec::new();
  for (var, ty) in vars {
    if let Some(v) = b_links.get(var) {
      let mut logs = Vec::new();
      for link in v.keys() {
        let camel = upper_camel(link);
        logs.push(quote!{Self::LoGMirrorEnum::#var(<#ty as ttgraph::TypedNode>::LoGMirror::#camel)});
      }

      links_arms.push(quote! {
        Self::#var(x) => {
          [#(#logs),*].into_iter().flat_map(|x|Self::expand_link_groups(x)).map(|x|
            (Vec::from_iter(self.iter_links(x)), self.get_bidiretional_link_mirrors_of(x))
          ).collect()
        },
      });
    }
  }

  quote! {
    fn get_bidiretional_links(&self) -> ttgraph::BidirectionalLinks<Self::LinkMirrorEnum> {
      match self {
        #(#links_arms)*
        _ => vec![],
      }
    }

    fn get_bidiretional_link_mirrors_of_log(&self, link: Self::LoGMirrorEnum) -> Vec<Self::LinkMirrorEnum> {
      match link {
        #(#link_mirrors_of_arms)*
        _ => vec![],
      }
    }
  }
}
