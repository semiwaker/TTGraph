use std::collections::{BTreeMap, BTreeSet};

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

fn expand_group(annotations: Vec<TypeAnnotation>, group_map: &BTreeMap<Ident, Vec<Ident>>) -> Vec<TypeAnnotation> {
  let mut result = Vec::new();
  for TypeAnnotation{var, link ,var2} in annotations {
    let mut expanded_var2 = Vec::new();
    for v2 in var2 {
      if let Some(x) = group_map.get(&v2) {
        expanded_var2.extend(x.clone());
      } else {
        expanded_var2.push(v2);
      }
    }

    if let Some(x) = group_map.get(&var) {
      for v in x {
        result.push(TypeAnnotation{var: v.clone(), link:link.clone(), var2:expanded_var2.clone()});
      }
    } else {
      result.push(TypeAnnotation{var, link ,var2: expanded_var2} );
    }
  }

  result
}

pub(crate) fn make_check_link_type(
  vars: &[(Ident, Type)], annotations: Vec<TypeAnnotation>, groups: &[NamedGroup],
) -> TokenStream {
  let mut arms = Vec::new();
  let mut anno_map: BTreeMap<Ident, BTreeSet<(Ident, Vec<Ident>)>> = BTreeMap::new();
  let mut group_map: BTreeMap<Ident, Vec<Ident>> = BTreeMap::new();
  for NamedGroup { name, idents } in groups {
    group_map.insert(name.clone(), idents.clone());
  }
  let annotations = expand_group(annotations, &group_map);
  for TypeAnnotation { var, link, var2 } in annotations {
    let camel = upper_camel(&link);
    anno_map.entry(var.clone()).or_default().insert((camel, var2));
  }
  for (var, ty) in vars {
    if let Some(vs) = anno_map.get(var) {
      let mut link_arms = Vec::new();
      for (link, var2) in vs {
        let mut var2_arms = Vec::new();
        let mut expect = Vec::new();
        for v2 in var2 {
          var2_arms.push(quote! {Self::NodeTypeMirror::#v2});
          expect.push(quote! {Self::NodeTypeMirror::#v2});
        }

        link_arms.push(quote! {
          <#ty as TypedNode>::LoGMirror::#link => match target {
            #(#var2_arms)|* => Ok(()),
            other => Err(ttgraph::LinkTypeError{
              link,
              expect: &[#(#expect),*],
              found: other,
            }),
          },
        })
      }
      arms.push(quote! {Self::LoGMirrorEnum::#var(v) => match v{
        #(#link_arms)*
        _ => Ok(()),
      },});
    } else {
      arms.push(quote! {Self::LoGMirrorEnum::#var(_) => Ok(()),});
    }
  }
  quote! {
    fn check_link_type_by_group(target: Self::NodeTypeMirror, link: Self::LoGMirrorEnum) -> ttgraph::LinkTypeCheckResult<Self> {
      match link {
        #(#arms)*
      }
    }
  }
}
