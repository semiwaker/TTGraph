use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{Generics, Ident, TypePath};

use std::collections::BTreeMap;

use super::group::*;
use super::upper_camel;

#[derive(Debug, Clone)]
pub(crate) enum LinkType {
  Direct(Ident, Ident),
  Set(Ident, Ident),
  Vec(Ident, Ident),
  Empty,
}

pub(crate) fn make_node_source_enum(result: &mut TokenStream, links: &Vec<LinkType>, name: &Ident) -> Ident {
  let source_enum = format_ident!("{}Source", name);
  let link_mirror = format_ident!("{}LinkMirror", name);
  let mut vars = Vec::new();
  for s in links {
    match &s {
      LinkType::Direct(_, camel) => vars.push(quote! {#camel}),
      LinkType::Set(_, camel) => vars.push(quote! {#camel}),
      LinkType::Vec(_, camel) => vars.push(quote! {#camel(usize)}),
      // LinkType::Enum(_, camel) => vars.push(quote! {#camel}),
      LinkType::Empty => vars.push(quote! {Empty}),
    }
  }

  let mut to_link_arms = Vec::new();
  for s in links {
    to_link_arms.push(match &s {
      LinkType::Direct(_, camel) => quote! {Self::#camel => #link_mirror::#camel,},
      LinkType::Set(_, camel) => quote! {Self::#camel => #link_mirror::#camel,},
      LinkType::Vec(_, camel) => quote! {Self::#camel(_) => #link_mirror::#camel,},
      LinkType::Empty => quote! {Self::Empty => #link_mirror::Empty,},
    })
  }

  quote! {
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub enum #source_enum{
      #(#vars),*
    }

    impl #source_enum{
      pub fn to_link_mirror(self) -> #link_mirror{
        match self{
          #(#to_link_arms)*
        }
      }
    }
  }
  .to_tokens(result);

  source_enum
}

pub(crate) fn make_link_mirror(result: &mut TokenStream, links: &Vec<LinkType>, name: &Ident) -> Ident {
  let source_enum = format_ident!("{}Source", name);
  let link_mirror = format_ident!("{}LinkMirror", name);
  let mut vars = Vec::new();
  for s in links {
    match &s {
      LinkType::Direct(_, camel) => vars.push(quote! {#camel}),
      LinkType::Set(_, camel) => vars.push(quote! {#camel}),
      LinkType::Vec(_, camel) => vars.push(quote! {#camel}),
      // LinkType::Enum(_, camel) => vars.push(quote! {#camel}),
      LinkType::Empty => vars.push(quote! {Empty}),
    }
  }

  let mut to_src_arms = Vec::new();
  for s in links {
    to_src_arms.push(match &s {
      LinkType::Direct(_, camel) => quote! {Self::#camel => #source_enum::#camel,},
      LinkType::Set(_, camel) => quote! {Self::#camel => #source_enum::#camel,},
      LinkType::Vec(_, camel) => quote! {Self::#camel => panic!("Vec type LinkMirror cannot be converted to Source!"),},
      LinkType::Empty => quote! {Self::Empty => #source_enum::Empty,},
    })
  }

  quote! {
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub enum #link_mirror{
      #(#vars),*
    }

    impl #link_mirror{
      pub fn to_source(self) -> #source_enum {
        match self {
          #(#to_src_arms)*
        }
      }
    }
  }
  .to_tokens(result);

  link_mirror
}

pub(crate) fn make_log_mirror(
  result: &mut TokenStream, links: &Vec<LinkType>, group_map: &BTreeMap<Ident, Vec<Ident>>, name: &Ident,
) -> Ident {
  let link_mirror = format_ident!("{}LinkMirror", name);
  let log_mirror = format_ident!("{}LoGMirror", name);
  let mut vars = Vec::new();
  let mut to_link_arms = Vec::new();
  for s in links {
    vars.push(match &s {
      LinkType::Direct(_, camel) => quote! {#camel},
      LinkType::Set(_, camel) => quote! {#camel},
      LinkType::Vec(_, camel) => quote! {#camel},
      // LinkType::Enum(_, camel) => vars.push(quote! {#camel}),
      LinkType::Empty => quote! {Empty},
    });
    to_link_arms.push(match &s {
      LinkType::Direct(_, camel) => quote! {Self::#camel => &[#link_mirror::#camel],},
      LinkType::Set(_, camel) => quote! {Self::#camel => &[#link_mirror::#camel],},
      LinkType::Vec(_, camel) => quote! {Self::#camel => &[#link_mirror::#camel],},
      LinkType::Empty => quote! {Self::Empty => &[#link_mirror::Empty],},
    });
  }
  for (g, v) in group_map {
    let camel = upper_camel(g);
    let mut links = Vec::new();
    for i in v {
      let c = upper_camel(i);
      links.push(quote! {#link_mirror::#c});
    }
    vars.push(quote! { #camel });
    to_link_arms.push(quote! { Self::#camel => &[#(#links),*], });
  }

  quote! {
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub enum #log_mirror{
      #(#vars),*
    }

    impl #log_mirror{
      pub fn to_links(self) -> &'static [#link_mirror] {
        match self {
          #(#to_link_arms)*
        }
      }
    }
  }
  .to_tokens(result);

  log_mirror
}

pub(crate) fn make_typed_node(
  links: &[LinkType], data: &[(Ident, TypePath)], groups: &[Vec<Ident>], group_map: &BTreeMap<Ident, Vec<Ident>>,
  name: &Ident, generics: &Generics, gen_mod: &Ident, source_enum: &Ident, link_mirror: &Ident, log_mirror: &Ident,
) -> TokenStream {
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

  let mut add_source_ops = Vec::new();

  // Add all sources into a vec for SourceIterator
  for s in links {
    match s {
      LinkType::Direct(ident, camel) => add_source_ops.push(quote! {
        if !self.#ident.is_empty() {
          sources.push((self.#ident, <#name as ttgraph::TypedNode>::Source::#camel));
        }
      }),
      LinkType::Set(ident, camel) => add_source_ops.push(quote! {
        for i in self.#ident.iter() {
          sources.push((*i, <#name as ttgraph::TypedNode>::Source::#camel));
        }
      }),
      LinkType::Vec(ident, camel) => add_source_ops.push(quote! {
        for (idx, i) in self.#ident.iter().enumerate() {
          if !i.is_empty() {
            sources.push((*i, <#name as ttgraph::TypedNode>::Source::#camel(idx)));
          }
        }
      }),
      LinkType::Empty => {},
    }
  }

  // Generate the match arms for iter_link()
  let mut iter_link_arms = Vec::new();
  for s in links {
    iter_link_arms.push(match s {
      LinkType::Direct(ident, camel) => quote! {
        Self::LinkMirror::#camel => if self.#ident.is_empty() {Box::new([].into_iter())} else {Box::new([self.#ident].into_iter())},
      },
      LinkType::Set(ident, camel) => quote! {
        Self::LinkMirror::#camel => Box::new(self.#ident.iter().map(|x|*x)),
      },
      LinkType::Vec(ident, camel) => quote! {
        Self::LinkMirror::#camel => Box::new(self.#ident.iter().filter(|x|!x.is_empty()).map(|x|*x)),
      },
      LinkType::Empty => quote! {
        Self::LinkMirror::Empty => Box::new([].into_iter()),
      },
    })
  }

  // Generate the match arms for modify_link()
  let mut modify_arms = Vec::new();
  for s in links {
    modify_arms.push(match s {
      LinkType::Direct(ident, camel) => quote! {
        Self::Source::#camel => {
          let replaced = self.#ident != new_idx;
          let removed = replaced && !self.#ident.is_empty();
          self.#ident = new_idx;
          (removed, replaced && !new_idx.is_empty())
        },
      },
      LinkType::Set(ident, camel) => quote! {
        Self::Source::#camel => {
          let removed = self.#ident.remove(&old_idx);
          let added = if !new_idx.is_empty() {
            self.#ident.insert(new_idx)
          } else {
            false
          };
          (removed, added)
        },
      },
      LinkType::Vec(ident, camel) => quote! {
        Self::Source::#camel(idx) => {
          let replaced = self.#ident[idx] != new_idx;
          let removed = replaced && !self.#ident[idx].is_empty();
          self.#ident[idx] = new_idx;
          (removed, replaced && !new_idx.is_empty())
        },
      },
      LinkType::Empty => quote! {
        Self::Source::Empty => (false, false),
      },
    })
  }

  // Generate the match arms for add_link()
  let mut add_link_arms = Vec::new();
  for s in links {
    add_link_arms.push(match s {
      LinkType::Direct(ident, camel) => quote! {
        Self::LinkMirror::#camel => {
          if self.#ident.is_empty() {
            if self.#ident != target{
              self.#ident = target;
              true
            } else{
              false
            }
          } else {
            assert!(self.#ident == target, "Add link on existing direct link");
            false
          }
        },
      },
      LinkType::Set(ident, camel) => quote! {
        Self::LinkMirror::#camel => {
          self.#ident.insert(target)
        },
      },
      LinkType::Vec(_, camel) => quote! {
        Self::LinkMirror::#camel => panic!("Add link on Vec<NodeIndex> is not supported!"),
      },
      LinkType::Empty => quote! {
        Self::LinkMirror::Empty => false,
      },
    })
  }

  // Generate the match arms for remove_link()
  let mut remove_link_arms = Vec::new();
  for s in links {
    remove_link_arms.push(match s {
      LinkType::Direct(ident, camel) => quote! {
        Self::LinkMirror::#camel => {
          if self.#ident.is_empty() {
            false
          } else {
            if self.#ident == target {
              self.#ident = ttgraph::NodeIndex::empty();
              true
            } else {
              false
            }
          }
        },
      },
      LinkType::Set(ident, camel) => quote! {
        Self::LinkMirror::#camel => {
          self.#ident.remove(&target)
        },
      },
      LinkType::Vec(_, camel) => quote! {
        Self::LinkMirror::#camel => panic!("Remove link on Vec<NodeIndex> is not supported!"),
      },
      LinkType::Empty => quote! {
        Self::LinkMirror::Empty => false,
      },
    })
  }

  // Generate the match arms for contains_link()
  let mut contains_link_arms = Vec::new();
  for s in links {
    contains_link_arms.push(match s {
      LinkType::Direct(ident, camel) => quote! {
        Self::LinkMirror::#camel => {
          if self.#ident.is_empty() {
            false
          } else {
            self.#ident == target
          }
        },
      },
      LinkType::Set(ident, camel) => quote! {
        Self::LinkMirror::#camel => {
          self.#ident.contains(&target)
        },
      },
      LinkType::Vec(ident, camel) => quote! {
        Self::LinkMirror::#camel => self.#ident.iter().any(|&x|x==target),
      },
      LinkType::Empty => quote! {
        Self::LinkMirror::Empty => false,
      },
    })
  }

  // Generate the static link type vec
  let mut link_type_vec = Vec::new();
  for s in links {
    match s {
      LinkType::Direct(..) => link_type_vec.push(quote! {ttgraph::LinkType::Point}),
      LinkType::Set(..) => link_type_vec.push(quote! {ttgraph::LinkType::Set}),
      LinkType::Vec(..) => link_type_vec.push(quote! {ttgraph::LinkType::Vec}),
      _ => {},
    }
  }

  // Generate the static link mirror vec
  let mut link_mirror_vec = Vec::new();
  for s in links {
    match s {
      LinkType::Direct(_, camel) => link_mirror_vec.push(quote! {Self::LinkMirror::#camel}),
      LinkType::Set(_, camel) => link_mirror_vec.push(quote! {Self::LinkMirror::#camel}),
      LinkType::Vec(_, camel) => link_mirror_vec.push(quote! {Self::LinkMirror::#camel}),
      _ => {},
    }
  }

  // Generate the static link name vec
  let mut link_name_vec = Vec::new();
  for s in links {
    match s {
      LinkType::Direct(name, _) => link_name_vec.push(quote! {std::stringify!(#name)}),
      LinkType::Set(name, _) => link_name_vec.push(quote! {std::stringify!(#name)}),
      LinkType::Vec(name, _) => link_name_vec.push(quote! {std::stringify!(#name)}),
      _ => {},
    }
  }

  let mut get_link_by_name_vec = Vec::new();
  for s in links {
    get_link_by_name_vec.push(match s {
      LinkType::Direct(name, camel) => {
        quote! {std::stringify!(#name) => self.iter_links(Self::LinkMirror::#camel),}
      },
      LinkType::Set(name, camel) => {
        quote! {std::stringify!(#name) => self.iter_links(Self::LinkMirror::#camel),}
      },
      LinkType::Vec(name, camel) => {
        quote! {std::stringify!(#name) => self.iter_links(Self::LinkMirror::#camel),}
      },
      _ => quote! {std::stringify!(#name) => Box::new([].into_iter()),},
    });
  }

  let get_links_by_group = make_get_links_by_group(links, groups);
  let get_log_by_name = make_get_link_or_group(links, group_map);

  // Generate the static data type vec
  let mut data_type_vec = Vec::new();
  for (_, ty) in data {
    data_type_vec.push(quote! {std::any::TypeId::of::<#ty>()});
  }

  // Generate the static data name vec
  let mut data_name_vec = Vec::new();
  for (ident, _) in data {
    data_name_vec.push(quote! {std::stringify!(#ident)});
  }

  // Generate the static data ref match arms
  let mut data_ref_arms = Vec::new();
  for (ident, _) in data {
    data_ref_arms.push(quote! {
      std::stringify!(#ident) => <dyn std::any::Any>::downcast_ref::<TGDataRefT>(&self.#ident),
    });
  }

  let mut to_log_arms = Vec::new();
  for (s, gs) in links.iter().zip(groups.iter()) {
    let mut logs = Vec::new();
    for g in gs {
      let c = upper_camel(g);
      logs.push(quote! {Self::LoGMirror::#c});
    }
    to_log_arms.push(match s {
      LinkType::Direct(_, camel) => quote! {Self::LinkMirror::#camel => &[Self::LoGMirror::#camel, #(#logs),*], },
      LinkType::Set(_, camel) => quote! {Self::LinkMirror::#camel => &[Self::LoGMirror::#camel, #(#logs),*], },
      LinkType::Vec(_, camel) => quote! {Self::LinkMirror::#camel => &[Self::LoGMirror::#camel, #(#logs),*], },
      LinkType::Empty => quote! {Self::LinkMirror::Empty => &[],},
    })
  }

  quote! {
    #[automatically_derived]
    impl #impl_generics ttgraph::TypedNode for #name #ty_generics #where_clause {
      type Source = #gen_mod::#source_enum;
      type LinkMirror = #gen_mod::#link_mirror;
      type LoGMirror = #gen_mod::#log_mirror;
      fn iter_sources(&self) -> std::vec::IntoIter<(ttgraph::NodeIndex, Self::Source)>  {
        let mut sources = Vec::new();
        #(#add_source_ops)*
        sources.into_iter()
      }
      fn iter_links(&self, link: Self::LinkMirror) -> Box<dyn std::iter::Iterator<Item = ttgraph::NodeIndex> + '_> {
        match link{
          #(#iter_link_arms)*
        }
      }
      fn modify_link(&mut self, source: Self::Source, old_idx:ttgraph::NodeIndex, new_idx: ttgraph::NodeIndex) -> (bool, bool) {
        match source{
          #(#modify_arms)*
        }
      }
      fn add_link(&mut self, link: Self::LinkMirror, target: ttgraph::NodeIndex) -> bool {
        match link{
          #(#add_link_arms)*
        }
      }
      fn remove_link(&mut self, link: Self::LinkMirror, target: ttgraph::NodeIndex) -> bool {
        match link{
          #(#remove_link_arms)*
        }
      }
      fn contains_link(&self, link: Self::LinkMirror, target: ttgraph::NodeIndex) -> bool {
        match link{
          #(#contains_link_arms)*
        }
      }

      fn link_types() -> &'static [ttgraph::LinkType] {
        &[#(#link_type_vec),*]
      }
      fn link_mirrors() -> &'static [Self::LinkMirror] {
        &[#(#link_mirror_vec),*]
      }
      fn link_names() -> &'static [&'static str] {
        &[#(#link_name_vec),*]
      }
      fn get_links_by_name(&self, name: &'static str) -> Box<dyn std::iter::Iterator<Item = ttgraph::NodeIndex> + '_> {
        match name {
          #(#get_link_by_name_vec)*
          _ => Box::new([].into_iter())
        }
      }
      #get_links_by_group
      #get_log_by_name

      // fn data_types() -> [std::any::TypeId] {
      //   [#(#data_type_vec),*]
      // }
      fn data_names() -> &'static [&'static str] {
        &[#(#data_name_vec),*]
      }
      fn data_ref_by_name<TGDataRefT:std::any::Any>(&self, name: &'static str) -> Option<&TGDataRefT> {
        match name {
          #(#data_ref_arms)*
          _ => None
        }
      }

      fn to_source(input: Self::LinkMirror) -> Self::Source {
        input.to_source()
      }
      fn to_link_mirror(input: Self::Source) -> Self::LinkMirror {
        input.to_link_mirror()
      }

      fn to_link_or_groups(input: Self::LinkMirror) -> &'static [Self::LoGMirror] {
        match input {
          #(#to_log_arms)*
        }
      }
    }
  }
}
