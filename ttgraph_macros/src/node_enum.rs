use proc_macro2::{self, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Generics, Ident, Type};

use crate::bidirectional::*;
use crate::group::*;
use crate::link_check::*;

pub(crate) fn make_source_enum(
  result: &mut TokenStream, generics: &Generics, vars: &Vec<(Ident, Type)>, enumt: &Ident,
) -> Ident {
  let source_enum = format_ident!("{}SourceEnum", enumt);

  let mut v = Vec::new();
  for (ident, ty) in vars {
    v.push(quote! {#ident(<self::super::#ty as ttgraph::TypedNode>::Source),});
  }

  quote! {
    #[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
    pub enum #source_enum #generics{
      #(#v)*
    }
  }
  .to_tokens(result);

  source_enum
}

pub(crate) fn make_link_mirror_enum(
  result: &mut TokenStream, generics: &Generics, vars: &Vec<(Ident, Type)>, enumt: &Ident,
) -> Ident {
  let link_mirror_enum = format_ident!("{}LinkMirrorEnum", enumt);
  let mut v = Vec::new();
  for (ident, ty) in vars {
    v.push(quote! {#ident(<self::super::#ty as ttgraph::TypedNode>::LinkMirror),});
  }

  quote! {
    #[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
    pub enum #link_mirror_enum #generics{
      #(#v)*
    }
  }
  .to_tokens(result);

  link_mirror_enum
}

pub(crate) fn make_log_mirror_enum(
  result: &mut TokenStream, generics: &Generics, vars: &Vec<(Ident, Type)>, enumt: &Ident,
) -> Ident {
  let log_mirror_enum = format_ident!("{}LoGMirrorEnum", enumt);
  let mut v = Vec::new();
  for (ident, ty) in vars {
    v.push(quote! {#ident(<self::super::#ty as ttgraph::TypedNode>::LoGMirror),});
  }

  quote! {
    #[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
    pub enum #log_mirror_enum #generics{
      #(#v)*
    }
  }
  .to_tokens(result);

  log_mirror_enum
}

pub(crate) fn make_node_type_mirror_enum(result: &mut TokenStream, vars: &Vec<(Ident, Type)>, enumt: &Ident) -> Ident {
  let enum_name = format_ident!("{}NodeTypeMirror", enumt);
  let mut v = Vec::new();
  for (ident, _) in vars {
    v.push(quote! {#ident,});
  }

  quote! {
    #[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
    pub enum #enum_name{
      #(#v)*
    }
  }
  .to_tokens(result);

  enum_name
}

pub(crate) fn make_node_enum(
  result: &mut TokenStream, generics: &Generics, vars: &Vec<(Ident, Type)>, enumt: &Ident, source_enum: &Ident,
  link_mirror_enum: &Ident, log_mirror_enum: &Ident, node_type_mirror: &Ident, gen_mod: &Ident, node_index: &Ident,
  discriminant: &Ident, bidirectional_links: &[BidirectionalLink], groups: &[NamedGroup],
  type_annotations: Vec<TypeAnnotation>,
) {
  let mut get_node_type_arms = Vec::new();
  for (ident, _) in vars {
    get_node_type_arms.push(quote! {Self::#ident(_) => Self::NodeTypeMirror::#ident,})
  }

  let mut iter_src_arms = Vec::new();
  for (ident, ty) in vars {
    iter_src_arms.push(quote! { Self::#ident(x) => Box::new(
      <#ty as TypedNode>::iter_sources(&x).map(|(idx, src)| (idx, Self::SourceEnum::#ident(src)))
      ),
    });
  }

  let mut iter_link_arms = Vec::new();
  for (ident, ty) in vars {
    iter_link_arms.push(quote! { Self::#ident(x) => {
        if let Self::LinkMirrorEnum::#ident(l) = link {
          <#ty as TypedNode>::iter_links(&x, l)
        } else {
          panic!("Unmatched node type and link type!")
        }
      },
    });
  }

  let mut mod_arms = Vec::new();
  for (ident, ty) in vars {
    mod_arms.push(quote! {
      Self::#ident(x) => {
        if let Self::SourceEnum::#ident(src) = source {
          let (removed, added) = <#ty as TypedNode>::modify_link(x, src, old_idx, new_idx);
          ttgraph::ModifyResult {
            bd_link_mirrors: self.get_bidiretional_link_mirrors_of(Self::LinkMirrorEnum::#ident(src.to_link_mirror())),
            added,
            removed,
          }
        } else {
          panic!("Unmatched node type and source type!")
        }
      },
    })
  }

  let mut add_link_arms = Vec::new();
  for (ident, ty) in vars {
    add_link_arms.push(quote! {
      Self::#ident(x) => {
        if let Self::LinkMirrorEnum::#ident(src) = link {
          <#ty as TypedNode>::add_link(x, src, target)
        } else {
          false
        }
      },
    })
  }

  let mut remove_link_arms = Vec::new();
  for (ident, ty) in vars {
    remove_link_arms.push(quote! {
      Self::#ident(x) => {
        if let Self::LinkMirrorEnum::#ident(src) = link {
          <#ty as TypedNode>::remove_link(x, src, target)
        } else {
          false
        }
      },
    })
  }

  let mut check_link_arms = Vec::new();
  for (ident, _) in vars {
    check_link_arms.push(quote! {
      Self::#ident(x) => {
        if let Self::LinkMirrorEnum::#ident(src) = link {
          true
        } else {
          false
        }
      },
    })
  }

  let mut contains_link_arms = Vec::new();
  for (ident, ty) in vars {
    contains_link_arms.push(quote! {
      Self::#ident(x) => {
        if let Self::LinkMirrorEnum::#ident(src) = link {
          <#ty as TypedNode>::contains_link(x, src, target)
        } else {
          false
        }
      },
    })
  }

  let mut get_link_by_name_arms = Vec::new();
  for (ident, _) in vars {
    get_link_by_name_arms.push(quote! {
      Self::#ident(x) => x.get_links_by_name(name),
    });
  }

  let mut get_link_by_group_arms = Vec::new();
  for (ident, _) in vars {
    get_link_by_group_arms.push(quote! {
      Self::#ident(x) => x.get_links_by_group(name),
    });
  }

  let mut data_ref_arms = Vec::new();
  for (ident, ty) in vars {
    data_ref_arms.push(quote! {
      Self::#ident(x) => <#ty as TypedNode>::data_ref_by_name(x, name),
    })
  }

  let mut to_src_arms = Vec::new();
  for (ident, _) in vars {
    to_src_arms.push(quote! {Self::LinkMirrorEnum::#ident(x) => Self::SourceEnum::#ident(x.to_source()), });
  }

  let mut to_link_arms = Vec::new();
  for (ident, _) in vars {
    to_link_arms.push(quote! {Self::SourceEnum::#ident(x) => Self::LinkMirrorEnum::#ident(x.to_link_mirror()), });
  }

  let mut to_log_arms = Vec::new();
  for (ident, ty) in vars {
    to_log_arms.push(quote! {Self::LinkMirrorEnum::#ident(x) => Vec::from_iter(<#ty as ttgraph::TypedNode>::to_link_or_groups(x).iter().map(|l|Self::LoGMirrorEnum::#ident(*l))), });
  }

  let mut expand_group_arms = Vec::new();
  for (ident, _) in vars {
    expand_group_arms.push(quote!{Self::LoGMirrorEnum::#ident(x) => Vec::from_iter(x.to_links().iter().map(|x|Self::LinkMirrorEnum::#ident(*x))),})
  }

  let mut match_bd_arms = Vec::new();
  for (ident, _) in vars {
    match_bd_arms.push(quote! {
      Self::#ident(_) => for l in links {
        if let Self::LinkMirrorEnum::#ident(_) = l {
          result.push(l);
        }
      },
    })
  }

  let mut disc_arms = Vec::new();
  for (ident, _) in vars {
    disc_arms.push(quote! { Self::#ident(_) => #discriminant::#ident })
  }

  let bidirectional_link = make_bidirectional_link(vars, bidirectional_links);
  let in_group = make_in_group(groups);
  let link_check = make_check_link_type(vars, type_annotations, groups);

  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
  quote!{
    #[automatically_derived]
    impl #impl_generics NodeEnum for #enumt #ty_generics #where_clause {
      type SourceEnum = #gen_mod::#source_enum #ty_generics;
      type LinkMirrorEnum = #gen_mod::#link_mirror_enum #ty_generics;
      type LoGMirrorEnum = #gen_mod::#log_mirror_enum #ty_generics;
      type NodeTypeMirror = #gen_mod::#node_type_mirror;
      fn get_node_type_mirror(&self) -> Self::NodeTypeMirror {
        match self{
          #(#get_node_type_arms)*
        }
      }
      fn iter_sources(&self) -> Box<dyn Iterator<Item = (NodeIndex, Self::SourceEnum)>> {
        match self {
          #(#iter_src_arms)*
        }
      }
      fn iter_links(&self, link: Self::LinkMirrorEnum) -> Box<dyn Iterator<Item = NodeIndex> + '_> {
        match self {
          #(#iter_link_arms)*
        }
      }
      fn modify_link(&mut self, source: Self::SourceEnum, old_idx: ttgraph::NodeIndex, new_idx: ttgraph::NodeIndex) -> ttgraph::ModifyResult<Self::LinkMirrorEnum> {
        match self{
          #(#mod_arms)*
        }
      }
      fn add_link(&mut self, link: Self::LinkMirrorEnum, target: ttgraph::NodeIndex) -> bool {
        match self{
          #(#add_link_arms)*
        }
      }
      fn remove_link(&mut self, link: Self::LinkMirrorEnum, target: ttgraph::NodeIndex) -> bool {
        match self{
          #(#remove_link_arms)*
        }
      }
      fn check_link(&self, link: Self::LinkMirrorEnum) -> bool {
        match self{
          #(#check_link_arms)*
        }
      }
      fn contains_link(&self, link: Self::LinkMirrorEnum, target: ttgraph::NodeIndex) -> bool {
        match self{
          #(#contains_link_arms)*
        }
      }
      fn get_links_by_name(&self, name: &'static str) -> Box<dyn std::iter::Iterator<Item = ttgraph::NodeIndex> + '_> {
        match self{
          #(#get_link_by_name_arms)*
        }
      }
      fn get_links_by_group(&self, name: &'static str) -> Vec<ttgraph::NodeIndex>{
        match self{
          #(#get_link_by_group_arms)*
        }
      }

      #in_group

      fn data_ref_by_name<T: std::any::Any>(&self, name: &'static str) -> Option<&T> {
        match self{
          #(#data_ref_arms)*
        }
      }

      fn to_link_mirror_enum(input: Self::SourceEnum) -> Self::LinkMirrorEnum {
        match input {
          #(#to_link_arms)*
        }
      }
      fn to_source_enum(input: Self::LinkMirrorEnum) -> Self::SourceEnum {
        match input {
          #(#to_src_arms)*
        }
      }
      fn to_log_mirror_enums(input: Self::LinkMirrorEnum) -> Vec<Self::LoGMirrorEnum> {
        match input {
          #(#to_log_arms)*
        }
      }
      fn expand_link_groups(input: Self::LoGMirrorEnum) -> Vec<Self::LinkMirrorEnum> {
        match input {
          #(#expand_group_arms)*
        }
      }

      #bidirectional_link

      #link_check

      fn match_bd_link_group(&self, links: Vec<Self::LinkMirrorEnum>) -> Vec<Self::LinkMirrorEnum> {
        let mut result = Vec::new();
        match self {
          #(#match_bd_arms)*
        }
        result
      }
    }

    #[automatically_derived]
    impl ttgraph::CateNode for #enumt {
      type D = #discriminant;
      type Index = #node_index;
    }

    #[automatically_derived]
    impl ttgraph::Discriminated<#discriminant> for #enumt {
      fn discriminant(&self) -> #discriminant {
        match self { #(#disc_arms),* }
      }
    }
  }.to_tokens(result);
}
