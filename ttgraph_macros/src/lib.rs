#![crate_type = "proc-macro"]

use std::collections::BTreeMap;
use proc_macro::TokenStream;
// use proc_macro2;
use proc_macro_error::*;
use quote::ToTokens;
use syn::{parse2, parse_macro_input, parse_quote, Fields, Item, ItemStruct, Type, Ident};

mod node_enum;
use node_enum::*;

mod typed_node;
use typed_node::*;

mod bidirectional;
use bidirectional::*;

mod link_check;
use link_check::*;

mod group;
use group::*;

mod utils;
use utils::*;

/// Collect TypedNodes together to form an enum
/// # Syntax
/// ```plain
/// node_enum!{
///   // rust enum
///   enum $EnumName{
///     // ...
///   }
///   // optional, to declare bidirectional links
///   bidirectional!{
///     $var.$field <-> $var.$field,
///     // ...
///   }
///   // optional, to declare the grouping of enum variant
///   group!{
///     $group_name{$var1, $var2, ...}
///     // ...
///   }
/// }
/// ```
#[proc_macro]
#[proc_macro_error]
pub fn node_enum(macro_input: TokenStream) -> TokenStream {
  let macro_input: syn::File = parse_macro_input!(macro_input);
  let Item::Enum(the_enum) = &macro_input.items[0] else {
    abort!(macro_input.items[0], "The node enum should be the first item of node_enum!")
  };
  let enumt = the_enum.ident.clone();
  let vis = the_enum.vis.clone();
  let generics = the_enum.generics.clone();

  let mut vars = Vec::new();
  for var in &the_enum.variants {
    let ident = var.ident.clone();
    if let Fields::Unnamed(f) = &var.fields {
      if f.unnamed.len() != 1 {
        emit_error! {f,
            "variants in node_enum should have only one unnamed field"
        };
      } else {
        vars.push((ident, f.unnamed.first().unwrap().ty.clone()));
      }
    } else {
      emit_error!(var, "variants in node_enum should have a node type as unnamed field");
    }
  }

  let mut result = proc_macro2::TokenStream::new();
  the_enum.to_tokens(&mut result);

  // if check_type_distinct(&vars) {
  //   for (ident, ty) in &vars {
  //     make_query_by_type_trait_impl(&mut result, &generics, &enumt, ident, ty);
  //     make_transaction_by_type_trait_impl(&mut result, &generics, &enumt, ident, ty);
  //   }
  // }
  let mut generated = proc_macro2::TokenStream::new();
  let source_enum = make_source_enum(&mut generated, &generics, &vars, &enumt);
  let link_mirror_enum = make_link_mirror_enum(&mut generated, &generics, &vars, &enumt);
  let log_mirror_enum = make_log_mirror_enum(&mut generated, &generics, &vars, &enumt);
  let node_type_mirror = make_node_type_mirror_enum(&mut generated, &vars, &enumt);

  let gen_mod = make_generated_mod(&mut result, generated, &enumt, &vis);
  
  let mut bidirectional_links = Vec::new();
  let mut groups = Vec::new();
  let mut type_annotations = Vec::new();
  for item in macro_input.items.iter().skip(1) {
    if let Item::Macro(the_macro) = item {
      if the_macro.mac.path.is_ident("bidirectional") {
        if let Err(err) =
          get_bidirectional_links(the_macro.mac.tokens.clone(), &mut bidirectional_links)
        {
          emit_error!(err.span(), "{}", err);
        }
      } else if the_macro.mac.path.is_ident("group") {
        let result: syn::Result<NamedGroupVec> = parse2(the_macro.mac.tokens.clone());
        match result {
          Ok(x) => groups.extend(x.groups),
          Err(err) => {
            emit_error!(err.span(), "{}", err);
          },
        }
      } else if the_macro.mac.path.is_ident("link_type") {
        let result: syn::Result<TypeAnnotationVec> = parse2(the_macro.mac.tokens.clone());
        match result {
          Ok(x) => type_annotations.extend(x.annotations),
          Err(err) => {
            emit_error!(err.span(), "{}", err);
          },
        }
      } else {
        abort!(the_macro.mac.path, "Unsupported macro");
      }
    } else {
      abort!(item, "Unsupported item type");
    }
  }

  check_bidirectional_links(&vars, &bidirectional_links, &groups);
  abort_if_dirty();

  let bidirectional_links = expand_bidirectional_links(bidirectional_links, &groups);
  make_node_enum(
    &mut result,
    &generics,
    &vars,
    &enumt,
    &source_enum,
    &link_mirror_enum,
    &log_mirror_enum,
    &node_type_mirror,
    &gen_mod,
    &bidirectional_links,
    &groups,
    type_annotations,
  );

  result.into()
}

/// Automatically implements `TypedNode` trait for a struct.
/// Helpep attributes:
/// + `#[group(group1, group2, ...)]`: declare this field (must be links) is inside some groups
#[proc_macro_derive(TypedNode, attributes(group))]
#[proc_macro_error]
pub fn typed_node(input: TokenStream) -> TokenStream {
  let input: ItemStruct = parse_macro_input!(input);
  let name = input.ident.clone();
  let vis = input.vis.clone();
  let generics = input.generics.clone();

  let Fields::Named(fields) = &input.fields else { panic!("Impossible!") };
  let mut links = Vec::new();
  let mut data = Vec::new();
  let mut groups = Vec::new();
  let mut group_map: BTreeMap<Ident, Vec<Ident>> = BTreeMap::new();
  let direct_paths = vec![parse_quote!(ttgraph::NodeIndex), parse_quote!(NodeIndex)];
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
    let mut is_link = false;
    if let Type::Path(p) = &f.ty {
      if direct_paths.contains(p) {
        links.push(LinkType::Direct(ident.clone(), upper_camel(&ident)));
        is_link = true;
      } else if hset_paths.contains(p) {
        links.push(LinkType::HSet(ident.clone(), upper_camel(&ident)));
        is_link = true;
      } else if bset_paths.contains(p) {
        links.push(LinkType::BSet(ident.clone(), upper_camel(&ident)));
        is_link = true;
      } else if vec_paths.contains(p) {
        links.push(LinkType::Vec(ident.clone(), upper_camel(&ident)));
        is_link = true;
      } else {
        data.push((ident.clone(), p.clone()));
      }
    }
    let mut have_group = false;
    for attr in &f.attrs {
      if attr.path().is_ident("group") {
        let mut cur_group = Vec::new();
        if let Err(err) = attr.parse_nested_meta(|meta| {
          if let Some(ident) = meta.path.get_ident() {
            cur_group.push(ident.clone());
            Ok(())
          } else {
            Err(meta.error("Expect identifier"))
          }
        }) {
          emit_error!(err.span(), "{}", err);
        }
        if !is_link {
          emit_error!(attr, "Can not group a non-link field!");
        } else {
          for g in &cur_group {
            group_map.entry(g.clone()).or_default().push(ident.clone());
          }
          groups.push(cur_group);
        }
        have_group = true;
      }
    }

    if !have_group && is_link {
      groups.push(Vec::new());
    }
  }
  if links.is_empty() {
    links.push(LinkType::Empty);
    groups.push(Vec::new());
  }

  let mut result = proc_macro2::TokenStream::new();
  let mut generated = proc_macro2::TokenStream::new();

  let source_enum = make_node_source_enum(&mut generated, &links, &name,);
  let link_mirror = make_link_mirror(&mut generated, &links, &name, );
  let log_mirror = make_log_mirror(&mut generated, &links, &group_map, &name);

  let gen_mod = make_generated_mod(&mut result, generated, &name, &vis);
  make_typed_node(
    &links,
    &data,
    &groups,
    &name,
    &generics,
    &gen_mod,
    &source_enum,
    &link_mirror,
    &log_mirror,
  ).to_tokens(&mut result);

  result.into()
}

// #[proc_macro]
// #[proc_macro_error]
// pub fn link_group(input: TokenStream) -> TokenStream {
//   let parse_result: syn::Result<NamedGroupVec> = parse(input);
//   let groups = match parse_result {
//     Ok(x) => x,
//     Err(err) => {
//       abort!(err.span(), "{}", err);
//     },
//   };
//   let mut result = proc_macro2::TokenStream::new();
//   make_get_links_by_group(groups).to_tokens(&mut result);

//   result.into()
// }

// #[proc_macro_derive(IndexEnum)]
// #[proc_macro_error]
// pub fn node_index_enum(input: TokenStream) -> TokenStream {
//   let input: ItemEnum = parse_macro_input!(input);
//   let name = input.ident.clone();
//   let vis = input.vis.clone();

//   let mut vars = Vec::new();
//   for var in &input.variants {
//     let ident = var.ident.clone();
//     if let Fields::Unnamed(f) = &var.fields {
//       if f.unnamed.len() != 1 {
//         emit_error! {f,
//             "variants in index_enum should have only one unnamed field"
//         };
//       } else {
//         vars.push(ident);
//       }
//     } else {
//       emit_error!(var, "variants in index_enum should have a node type as unnamed field");
//     }
//   }

//   let mut result = proc_macro2::TokenStream::new();
//   make_index_enum_trait(&mut result, &vars, &name, &vis);

//   result.into()
// }
