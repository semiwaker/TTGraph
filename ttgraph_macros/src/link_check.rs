use std::collections::{btree_map, BTreeMap, BTreeSet};

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{self, braced, token, Ident, Token, Type};

use crate::group::NamedGroup;
use crate::utils::upper_camel;

pub(crate) struct TypeAnnotation {
  pub var: Ident,
  pub link: Ident,
  pub var2: Vec<Ident>,
}

pub(crate) struct TypeAnnotationVec {
  pub annotations: Vec<TypeAnnotation>,
}

impl Parse for TypeAnnotation {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let var = input.parse()?;
    let _: Token![.] = input.parse()?;
    let link = input.parse()?;
    let _: Token![:] = input.parse()?;
    let var2 = if input.peek(token::Brace) {
      let content;
      let _ = braced!(content in input);
      let var2 = content.parse_terminated(Ident::parse, Token![,])?;
      var2.into_iter().collect()
    } else {
      Vec::from([input.parse()?])
    };
    Ok(TypeAnnotation { var, link, var2 })
  }
}

impl Parse for TypeAnnotationVec {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let result = input.parse_terminated(TypeAnnotation::parse, Token![,])?;
    Ok(TypeAnnotationVec {
      annotations: result.into_iter().collect(),
    })
  }
}

pub(crate) fn make_check_link_type(
  vars: &[(Ident, Type)], annotations: &[TypeAnnotation], groups: &[NamedGroup],
) -> TokenStream {
  let mut arms = Vec::new();
  let mut anno_map: BTreeMap<Ident, BTreeSet<(Ident, Vec<Ident>)>> = BTreeMap::new();
  let mut group_map: BTreeMap<Ident, Vec<Ident>> = BTreeMap::new();
  for NamedGroup { name, idents } in groups {
    group_map.insert(name.clone(), idents.clone());
  }
  for TypeAnnotation { var, link, var2 } in annotations {
    let camel = upper_camel(link);
    let mut legal_types = Vec::new();
    for v in var2 {
      if let btree_map::Entry::Occupied(g) = group_map.entry(v.clone()) {
        legal_types.extend(g.get().clone().into_iter());
      } else {
        legal_types.push(v.clone());
      }
    }
    anno_map.entry(var.clone()).or_default().insert((camel, legal_types));
  }
  for (var, ty) in vars {
    if let btree_map::Entry::Occupied(v) = anno_map.entry(var.clone()) {
      let mut link_arms = Vec::new();
      for (link, var2) in v.get() {
        let mut var2_arms = Vec::new();
        for v2 in var2 {
          var2_arms.push(quote! {Self::NodeTypeMirror::#v2 => true,})
        }
        link_arms.push(quote! {
          <#ty as TypedNode>::LinkMirror::#link => match target {
            #(#var2_arms)*
            _ => false,
          },
        })
      }
      arms.push(quote! {Self::LinkMirrorEnum::#var(v) => match v{
        #(#link_arms)*
        _ => true,
      },});
    } else {
      arms.push(quote! {Self::LinkMirrorEnum::#var(_) => true,});
    }
  }
  quote! {
    fn check_link_type(target: Self::NodeTypeMirror, link: Self::LinkMirrorEnum) -> bool {
      match link {
        #(#arms)*
      }
    }
  }
}
