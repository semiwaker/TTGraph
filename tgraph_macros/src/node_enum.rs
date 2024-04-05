use proc_macro2::{self, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Ident, Type, Visibility};

use crate::bidirectional::*;

pub fn make_gentrait_impl(
  result: &mut TokenStream, enumt: &Ident, ident: &Ident, ty: &Type, trait_ident: &Ident,
  vis: &Visibility,
) {
  let iter_ident = format_ident!("TGGenIter{}", ident);

  quote! {
    impl<'a> #trait_ident<'a, #iter_ident<'a>> for #ty{
      fn iter_by_type(graph: &'a tgraph::typed_graph::Graph<#enumt>) -> #iter_ident<'a>{
        #iter_ident{
          it: graph.iter_nodes(),
        }
      }
      fn get_by_type<'b>(graph: &'b tgraph::typed_graph::Graph<#enumt>, idx: tgraph::typed_graph::NodeIndex) -> Option<&#ty>{
        graph.get_node(idx).and_then(|x| if let #enumt::#ident(y) = x { Some(y) } else { None })
      }
    }

    #vis struct #iter_ident<'a> {
      it: tgraph::arena::Iter<'a, tgraph::typed_graph::NodeIndex, #enumt>
    }
    impl<'a> std::iter::Iterator for #iter_ident<'a>{
      type Item = (NodeIndex, &'a #ty);
      fn next(&mut self) -> Option<Self::Item> {
        self.it.next().and_then(|(idx, node)|
          if let #enumt::#ident(x) = node {
            Some((*idx, x))
          } else {
            self.next()
          }
        )
      }
    }
    impl<'a> std::iter::FusedIterator for #iter_ident<'a>{}
  }.to_tokens(result);
}

pub fn make_source_enum(
  result: &mut TokenStream, vars: &Vec<(Ident, Type)>, enumt: &Ident, vis: &Visibility,
) -> Ident {
  let source_enum_name = format_ident!("{}SourceEnum", enumt);

  let mut v = Vec::new();
  for (ident, ty) in vars {
    v.push(quote! {#ident(<#ty as TypedNode>::Source),});
  }

  quote! {
    #[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
    #vis enum #source_enum_name{
      #(#v)*
    }
  }
  .to_tokens(result);

  source_enum_name
}

pub fn make_link_mirror_enum(
  result: &mut TokenStream, vars: &Vec<(Ident, Type)>, enumt: &Ident, vis: &Visibility,
) -> Ident {
  let link_mirror_enum_name = format_ident!("{}LinkMirrorEnum", enumt);
  let mut v = Vec::new();
  for (ident, ty) in vars {
    v.push(quote! {#ident(<#ty as TypedNode>::LinkMirror),});
  }

  quote! {
    #[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
    #vis enum #link_mirror_enum_name{
      #(#v)*
    }
  }
  .to_tokens(result);

  link_mirror_enum_name
}

pub fn make_node_type_mirror_enum(
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

pub fn make_node_enum(
  result: &mut TokenStream, vars: &Vec<(Ident, Type)>, enumt: &Ident,
  source_enum_name: &Ident, link_mirror_enum_name: &Ident, node_type_mirror_name: &Ident,
  bidirectional_links: &Vec<BidirectionalLink>,
) {
  let mut iter_src_arms = Vec::new();
  for (ident, ty) in vars {
    iter_src_arms.push(quote! { Self::#ident(x) => Box::new(
      <#ty as TypedNode>::iter_source(&x).map(|(idx, src)| (idx, Self::SourceEnum::#ident(src)))
      ),
    });
  }

  let mut iter_link_arms = Vec::new();
  for (ident, ty) in vars {
    iter_link_arms.push(quote! { Self::#ident(x) => {
        if let Self::LinkMirrorEnum::#ident(l) = link {
          <#ty as TypedNode>::iter_link(&x, l)
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
          <#ty as TypedNode>::modify_link(x, src, old_idx, new_idx);
          tgraph::typed_graph::BidirectionalSideEffect {
            link_mirrors: self.get_bidiretional_link_mirrors_of(Self::LinkMirrorEnum::#ident(src.to_link_mirror())),
            add: new_idx,
            remove: old_idx,
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

  quote!{
    impl NodeEnum for #enumt {
      type SourceEnum = #source_enum_name;
      type LinkMirrorEnum = #link_mirror_enum_name;
      type NodeTypeMirror = #node_type_mirror_name;
      fn iter_source(&self) -> Box<dyn Iterator<Item = (NodeIndex, Self::SourceEnum)>> {
        match self {
          #(#iter_src_arms)*
        }
      }
      fn iter_link(&self, link: Self::LinkMirrorEnum) -> Box<dyn Iterator<Item = NodeIndex> + '_> {
        match self {
          #(#iter_link_arms)*
        }
      }
      fn modify_link(&mut self, source: Self::SourceEnum, old_idx: tgraph::typed_graph::NodeIndex, new_idx: tgraph::typed_graph::NodeIndex) -> tgraph::typed_graph::BidirectionalSideEffect<Self::LinkMirrorEnum> {
        match self{
          #(#mod_arms)*
        }
      }
      fn add_link(&mut self, link: Self::LinkMirrorEnum, target: tgraph::typed_graph::NodeIndex) -> bool {
        match self{
          #(#add_link_arms)*
        }
      }
      fn remove_link(&mut self, link: Self::LinkMirrorEnum, target: tgraph::typed_graph::NodeIndex) -> bool {
        match self{
          #(#remove_link_arms)*
        }
      }
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
