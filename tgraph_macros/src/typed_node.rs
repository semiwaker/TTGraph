use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_quote, Fields, Generics, Ident, ItemStruct, Type, TypePath, Visibility};

use crate::utils::*;

#[derive(Debug)]
pub(crate) enum LinkType {
  Direct(Ident, Ident),
  HSet(Ident, Ident),
  BSet(Ident, Ident),
  Vec(Ident, Ident),
  Empty,
}

pub(crate) fn get_members(input: &ItemStruct) -> (Vec<LinkType>, Vec<(Ident, TypePath)>) {
  let Fields::Named(fields) = &input.fields else { panic!("Impossible!") };
  // eprintln!("{:?}", fields.named);
  let mut links = Vec::new();
  let mut data = Vec::new();
  let direct_paths = vec![
    parse_quote!(tgraph::typed_graph::NodeIndex),
    parse_quote!(typed_graph::NodeIndex),
    parse_quote!(NodeIndex),
  ];
  let mut hset_paths = Vec::new();
  let mut bset_paths = Vec::new();
  let mut vec_paths = Vec::new();
  for dpath in &direct_paths {
    hset_paths.push(parse_quote!(std::collections::HashSet<#dpath>));
    hset_paths.push(parse_quote!(collections::HashSet<#dpath>));
    hset_paths.push(parse_quote!(HashSet<#dpath>));

    bset_paths.push(parse_quote!(std::collections::BTreeSet<#dpath>));
    bset_paths.push(parse_quote!(collections::BTreeSet<#dpath>));
    bset_paths.push(parse_quote!(BTreeSet<#dpath>));

    vec_paths.push(parse_quote!(std::vec::Vec<#dpath>));
    vec_paths.push(parse_quote!(vec::Vec<#dpath>));
    vec_paths.push(parse_quote!(Vec<#dpath>));
  }

  for f in &fields.named {
    let ident = f.ident.clone().unwrap();
    if let Type::Path(p) = &f.ty {
      if direct_paths.contains(p) {
        links.push(LinkType::Direct(ident.clone(), upper_camel(&ident)))
      } else if hset_paths.contains(p) {
        links.push(LinkType::HSet(ident.clone(), upper_camel(&ident)))
      } else if bset_paths.contains(p) {
        links.push(LinkType::BSet(ident.clone(), upper_camel(&ident)))
      } else if vec_paths.contains(p) {
        links.push(LinkType::Vec(ident.clone(), upper_camel(&ident)))
      } else {
        data.push((ident.clone(), p.clone()));
      }
      // } else if let PathArguments::AngleBracketed(a) =
      //   &p.path.segments.last().unwrap().arguments
      // {
      //   let path1 = parse_quote!(tgraph::typed_graph::NIEWrap #a);
      //   let path2 = parse_quote!(typed_graph::NIEWrap #a);
      //   let path3 = parse_quote!(NIEWrap #a);
      //   if p.path == path1 || p.path == path2 || p.path == path3 {
      //     result.push(LinkType::Enum(ident.clone(), upper_camel(&ident)))
      //   }
      // }
    }
  }
  if links.is_empty() {
    links.push(LinkType::Empty);
  }
  (links, data)
}

pub(crate) fn make_node_source_enum(
  result: &mut TokenStream, links: &Vec<LinkType>, name: &Ident, vis: &Visibility,
) -> Ident {
  let source_enum = format_ident!("{}Source", name);
  let link_mirror = format_ident!("{}LinkMirror", name);
  let mut vars = Vec::new();
  for s in links {
    match &s {
      LinkType::Direct(_, camel) => vars.push(quote! {#camel}),
      LinkType::HSet(_, camel) => vars.push(quote! {#camel}),
      LinkType::BSet(_, camel) => vars.push(quote! {#camel}),
      LinkType::Vec(_, camel) => vars.push(quote! {#camel(usize)}),
      // LinkType::Enum(_, camel) => vars.push(quote! {#camel}),
      LinkType::Empty => vars.push(quote! {Empty}),
    }
  }

  let mut to_link_arms = Vec::new();
  for s in links {
    to_link_arms.push(match &s {
      LinkType::Direct(_, camel) => quote! {Self::#camel => #link_mirror::#camel,},
      LinkType::HSet(_, camel) => quote! {Self::#camel => #link_mirror::#camel,},
      LinkType::BSet(_, camel) => quote! {Self::#camel => #link_mirror::#camel,},
      LinkType::Vec(_, camel) => quote! {Self::#camel(_) => #link_mirror::#camel,},
      LinkType::Empty => quote! {Self::Empty => #link_mirror::Empty,},
    })
  }

  quote! {
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    #vis enum #source_enum{
      #(#vars),*
    }

    impl #source_enum{
      pub fn to_link_mirror(&self) -> #link_mirror{
        match self{
          #(#to_link_arms)*
        }
      }
    }
  }
  .to_tokens(result);

  source_enum
}

pub(crate) fn make_link_mirror(
  result: &mut TokenStream, links: &Vec<LinkType>, name: &Ident, vis: &Visibility,
) -> Ident {
  let source_enum = format_ident!("{}Source", name);
  let link_mirror = format_ident!("{}LinkMirror", name);
  let mut vars = Vec::new();
  for s in links {
    match &s {
      LinkType::Direct(_, camel) => vars.push(quote! {#camel}),
      LinkType::HSet(_, camel) => vars.push(quote! {#camel}),
      LinkType::BSet(_, camel) => vars.push(quote! {#camel}),
      LinkType::Vec(_, camel) => vars.push(quote! {#camel}),
      // LinkType::Enum(_, camel) => vars.push(quote! {#camel}),
      LinkType::Empty => vars.push(quote! {Empty}),
    }
  }

  let mut to_src_arms = Vec::new();
  for s in links {
    to_src_arms.push(match &s {
      LinkType::Direct(_, camel) => quote! {Self::#camel => #source_enum::#camel,},
      LinkType::HSet(_, camel) => quote! {Self::#camel => #source_enum::#camel,},
      LinkType::BSet(_, camel) => quote! {Self::#camel => #source_enum::#camel,},
      LinkType::Vec(_, camel) => quote! {Self::#camel => panic!("Vec type LinkMirror cannot be converted to Source!"),},
      LinkType::Empty => quote! {Self::Empty => #source_enum::Empty,},
    })
  }

  quote! {
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    #vis enum #link_mirror{
      #(#vars),*
    }

    impl #link_mirror{
      pub fn to_source(&self) -> #source_enum {
        match self {
          #(#to_src_arms)*
        }
      }
    }
  }
  .to_tokens(result);

  link_mirror
}

pub(crate) fn make_typed_node(
  result: &mut TokenStream, sources: &Vec<LinkType>, data: &Vec<(Ident, TypePath)>,
  name: &Ident, vis: &Visibility, generics: &Generics, source_enum: &Ident,
  link_mirror: &Ident,
) {
  let iterator_ident = format_ident!("{}SourceIterator", name);
  let mut add_source_ops = Vec::new();
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

  // Add all sources into a vec for SourceIterator
  for s in sources {
    match s {
      LinkType::Direct(ident, camel) => add_source_ops.push(quote! {
        if !node.#ident.is_empty() {
          sources.push((node.#ident, #source_enum::#camel));
        }
      }),
      LinkType::HSet(ident, camel) => add_source_ops.push(quote! {
        for i in node.#ident.iter() {
          sources.push((*i, #source_enum::#camel));
        }
      }),
      LinkType::BSet(ident, camel) => add_source_ops.push(quote! {
        for i in node.#ident.iter() {
          sources.push((*i, #source_enum::#camel));
        }
      }),
      LinkType::Vec(ident, camel) => add_source_ops.push(quote! {
        for (idx, i) in node.#ident.iter().enumerate() {
          sources.push((*i, #source_enum::#camel(idx)));
        }
      }),
      // LinkType::Enum(ident, camel) => add_source_ops.push(quote! {
      //   sources.push((tgraph::typed_graph::IndexEnum::index(&node.#ident.value), #source_enum::#camel));
      // }),
      LinkType::Empty => {},
    }
  }

  // Generate the match arms for iter_link()
  let mut iter_link_arms = Vec::new();
  for s in sources {
    iter_link_arms.push(match s {
      LinkType::Direct(ident, camel) => quote! {
        Self::LinkMirror::#camel => if self.#ident.is_empty() {Box::new([].into_iter())} else {Box::new([self.#ident].into_iter())},
      },
      LinkType::HSet(ident, camel) => quote! {
        Self::LinkMirror::#camel => Box::new(self.#ident.iter().map(|x|*x)),
      },
      LinkType::BSet(ident, camel) => quote! {
        Self::LinkMirror::#camel => Box::new(self.#ident.iter().map(|x|*x)),
      },
      LinkType::Vec(ident, camel) => quote! {
        Self::LinkMirror::#camel => Box::new(self.#ident.iter().map(|x|*x)),
      },
      LinkType::Empty => quote! {
        Self::LinkMirror::Empty => Box::new([].into_iter()),
      },
    })
  }

  // Generate the match arms for modify_link()
  let mut modify_arms = Vec::new();
  for s in sources {
    modify_arms.push(match s {
      LinkType::Direct(ident, camel) => quote! {
        Self::Source::#camel => {
          let replaced = self.#ident != new_idx;
          let removed = replaced && !self.#ident.is_empty();
          self.#ident = new_idx;
          (removed, replaced && !new_idx.is_empty())
        },
      },
      LinkType::HSet(ident, camel) => quote! {
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
      LinkType::BSet(ident, camel) => quote! {
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
      // LinkType::Enum(ident, camel) => quote! {
      //   #source_enum::#camel => {
      //     tgraph::typed_graph::IndexEnum::modify(&mut self.#ident.value, new_idx);
      //   }
      // },
      LinkType::Empty => quote! {
        Self::Source::Empty => (false, false),
      },
    })
  }

  // Generate the match arms for add_link()
  let mut add_link_arms = Vec::new();
  for s in sources {
    add_link_arms.push(match s {
      LinkType::Direct(ident, camel) => quote!{
        Self::LinkMirror::#camel => {
          if self.#ident.is_empty() {
            if self.#ident != target{
              self.#ident = target;
              true
            } else{
              false
            }
          } else {
            assert!(self.#ident == target);
            false
          }
        },
      },
      LinkType::HSet(ident, camel) => quote!{
        Self::LinkMirror::#camel => {
          self.#ident.insert(target)
        },
      },
      LinkType::BSet(ident, camel) => quote!{
        Self::LinkMirror::#camel => {
          self.#ident.insert(target)
        },
      },
      LinkType::Vec(_, camel) => quote!{
        Self::LinkMirror::#camel => panic!("Add link on Vec<NodeIndex> is not supported!"),
      },
      LinkType::Empty => quote! {
        Self::LinkMirror::Empty => false,
      },
    })
  }

  // Generate the match arms for remove_link()
  let mut remove_link_arms = Vec::new();
  for s in sources {
    remove_link_arms.push(match s {
      LinkType::Direct(ident, camel) => quote!{
        Self::LinkMirror::#camel => {
          if self.#ident.is_empty() {
            false
          } else {
            if self.#ident == target {
              self.#ident = tgraph::typed_graph::NodeIndex::empty();
              true
            } else {
              false
            }
          }
        },
      },
      LinkType::HSet(ident, camel) => quote!{
        Self::LinkMirror::#camel => {
          self.#ident.remove(&target)
        },
      },
      LinkType::BSet(ident, camel) => quote!{
        Self::LinkMirror::#camel => {
          self.#ident.remove(&target)
        },
      },
      LinkType::Vec(_, camel) => quote!{
        Self::LinkMirror::#camel => panic!("Remove link on Vec<NodeIndex> is not supported!"),
      },
      LinkType::Empty => quote! {
        Self::LinkMirror::Empty => false,
      },
    })
  }

  // Generate the static link type vec
  let mut link_type_vec = Vec::new();
  for s in sources {
    match s {
      LinkType::Direct(..) => {
        link_type_vec.push(quote! {tgraph::typed_graph::LinkType::Point})
      },
      LinkType::HSet(..) => {
        link_type_vec.push(quote! {tgraph::typed_graph::LinkType::HSet})
      },
      LinkType::BSet(..) => {
        link_type_vec.push(quote! {tgraph::typed_graph::LinkType::BSet})
      },
      LinkType::Vec(..) => {
        link_type_vec.push(quote! {tgraph::typed_graph::LinkType::Vec})
      },
      _ => {},
    }
  }

  // Generate the static link mirror vec
  let mut link_mirror_vec = Vec::new();
  for s in sources {
    match s {
      LinkType::Direct(_, camel) => link_mirror_vec.push(quote! {#link_mirror::#camel}),
      LinkType::HSet(_, camel) => link_mirror_vec.push(quote! {#link_mirror::#camel}),
      LinkType::BSet(_, camel) => link_mirror_vec.push(quote! {#link_mirror::#camel}),
      LinkType::Vec(_, camel) => link_mirror_vec.push(quote! {#link_mirror::#camel}),
      _ => {},
    }
  }

  // Generate the static link name vec
  let mut link_name_vec = Vec::new();
  for s in sources {
    match s {
      LinkType::Direct(name, _) => link_name_vec.push(quote! {std::stringify!(#name)}),
      LinkType::HSet(name, _) => link_name_vec.push(quote! {std::stringify!(#name)}),
      LinkType::BSet(name, _) => link_name_vec.push(quote! {std::stringify!(#name)}),
      LinkType::Vec(name, _) => link_name_vec.push(quote! {std::stringify!(#name)}),
      _ => {},
    }
  }

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

  quote! {
    #vis struct #iterator_ident {
      sources: Vec<(NodeIndex, #source_enum)>,
      cur: usize
    }
    impl #impl_generics tgraph::typed_graph::SourceIterator<#name #ty_generics> for #iterator_ident #where_clause{
      type Source = #source_enum;
      fn new(node: &#name #ty_generics) -> Self{
        let mut sources = Vec::new();
        #(#add_source_ops)*
        #iterator_ident{ sources, cur: 0 }
      }
    }
    impl std::iter::Iterator for #iterator_ident {
      type Item = (NodeIndex, #source_enum);
      fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.sources.len() {
          None
        } else {
          let result = self.sources[self.cur].clone();
          self.cur += 1;
          Some(result)
        }
      }
    }
    impl #impl_generics tgraph::typed_graph::TypedNode for #name #ty_generics #where_clause {
      type Source = #source_enum;
      type LinkMirror = #link_mirror;
      type Iter = #iterator_ident;
      fn iter_source(&self) -> Self::Iter {
        #iterator_ident::new(&self)
      }
      fn iter_link(&self, link: Self::LinkMirror) -> Box<dyn std::iter::Iterator<Item = tgraph::typed_graph::NodeIndex> + '_> {
        match link{
          #(#iter_link_arms)*
        }
      }
      fn modify_link(&mut self, source: Self::Source, old_idx:tgraph::typed_graph::NodeIndex, new_idx: tgraph::typed_graph::NodeIndex) -> (bool, bool) {
        match source{
          #(#modify_arms)*
        }
      }
      fn add_link(&mut self, link: Self::LinkMirror, target: tgraph::typed_graph::NodeIndex) -> bool {
        match link{
          #(#add_link_arms)*
        }
      }
      fn remove_link(&mut self, link: Self::LinkMirror, target: tgraph::typed_graph::NodeIndex) -> bool {
        match link{
          #(#remove_link_arms)*
        }
      }

      fn link_types() -> &'static [tgraph::typed_graph::LinkType] {
        &[#(#link_type_vec),*]
      }
      fn link_mirrors() -> &'static [Self::LinkMirror] {
        &[#(#link_mirror_vec),*]
      }
      fn link_names() -> &'static [&'static str] {
        &[#(#link_name_vec),*]
      }

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

      fn to_source(input: &Self::LinkMirror) -> Self::Source {
        input.to_source()
      }
      fn to_link_mirror(input: &Self::Source) -> Self::LinkMirror {
        input.to_link_mirror()
      }
    }
  }
  .to_tokens(result);
}
