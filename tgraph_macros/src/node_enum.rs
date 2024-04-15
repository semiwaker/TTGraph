use proc_macro2::{self, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Generics, Ident, Type, Visibility};

use crate::bidirectional::*;
use crate::group::*;

pub(crate) fn make_source_enum(
  result: &mut TokenStream, generics: &Generics, vars: &Vec<(Ident, Type)>, enumt: &Ident, vis: &Visibility,
) -> Ident {
  let source_enum_name = format_ident!("{}SourceEnum", enumt);

  let mut v = Vec::new();
  for (ident, ty) in vars {
    v.push(quote! {#ident(<#ty as TypedNode>::Source),});
  }

  quote! {
    #[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
    #vis enum #source_enum_name #generics{
      #(#v)*
    }
  }
  .to_tokens(result);

  source_enum_name
}

pub(crate) fn make_link_mirror_enum(
  result: &mut TokenStream, generics: &Generics, vars: &Vec<(Ident, Type)>, enumt: &Ident, vis: &Visibility,
) -> Ident {
  let link_mirror_enum_name = format_ident!("{}LinkMirrorEnum", enumt);
  let mut v = Vec::new();
  for (ident, ty) in vars {
    v.push(quote! {#ident(<#ty as TypedNode>::LinkMirror),});
  }

  quote! {
    #[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
    #vis enum #link_mirror_enum_name #generics{
      #(#v)*
    }
  }
  .to_tokens(result);

  link_mirror_enum_name
}

pub(crate) fn make_node_type_mirror_enum(
  result: &mut TokenStream, vars: &Vec<(Ident, Type)>, enumt: &Ident, vis: &Visibility,
) -> Ident {
  let enum_name = format_ident!("{}NodeTypeMirror", enumt);
  let mut v = Vec::new();
  for (ident, _) in vars {
    v.push(quote! {#ident,});
  }

  quote! {
    #[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
    #vis enum #enum_name{
      #(#v)*
    }
  }
  .to_tokens(result);

  enum_name
}

pub(crate) fn make_node_enum(
  result: &mut TokenStream, generics: &Generics, vars: &Vec<(Ident, Type)>, enumt: &Ident,
  source_enum_name: &Ident, link_mirror_enum_name: &Ident, node_type_mirror_name: &Ident,
  bidirectional_links: &Vec<BidirectionalLink>, groups: &Vec<NamedGroup>
) {
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
          tgraph::BidirectionalSideEffect {
            link_mirrors: self.get_bidiretional_link_mirrors_of(Self::LinkMirrorEnum::#ident(src.to_link_mirror())),
            add: if (added) {new_idx} else {tgraph::NodeIndex::empty()},
            remove: if (removed) {old_idx} else {tgraph::NodeIndex::empty()},
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
    to_link_arms
      .push(quote! {Self::SourceEnum::#ident(x) => Self::LinkMirrorEnum::#ident(x.to_link_mirror()), });
  }

  let bidirectional_link = make_bidirectional_link(vars, bidirectional_links);
  let in_group = make_in_group(groups);

  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
  quote!{
    impl #impl_generics NodeEnum for #enumt #ty_generics #where_clause {
      type SourceEnum = #source_enum_name #ty_generics;
      type LinkMirrorEnum = #link_mirror_enum_name #ty_generics;
      type NodeTypeMirror = #node_type_mirror_name;
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
      fn modify_link(&mut self, source: Self::SourceEnum, old_idx: tgraph::NodeIndex, new_idx: tgraph::NodeIndex) -> tgraph::BidirectionalSideEffect<Self::LinkMirrorEnum> {
        match self{
          #(#mod_arms)*
        }
      }
      fn add_link(&mut self, link: Self::LinkMirrorEnum, target: tgraph::NodeIndex) -> bool {
        match self{
          #(#add_link_arms)*
        }
      }
      fn remove_link(&mut self, link: Self::LinkMirrorEnum, target: tgraph::NodeIndex) -> bool {
        match self{
          #(#remove_link_arms)*
        }
      }
      fn check_link(&self, link: Self::LinkMirrorEnum) -> bool {
        match self{
          #(#check_link_arms)*
        }
      }
      fn get_links_by_name(&self, name: &'static str) -> Box<dyn std::iter::Iterator<Item = tgraph::NodeIndex> + '_> {
        match self{
          #(#get_link_by_name_arms)*
        }
      }
      fn get_links_by_group(&self, name: &'static str) -> Vec<tgraph::NodeIndex>{
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
      
      fn to_link_mirror_enum(input: &Self::SourceEnum) -> Self::LinkMirrorEnum{
        match input {
          #(#to_link_arms)*
        }
      }
      fn to_source_enum(input: &Self::LinkMirrorEnum) -> Self::SourceEnum{
        match input {
          #(#to_src_arms)*
        }
      }

      #bidirectional_link
    }
  }.to_tokens(result);
}
