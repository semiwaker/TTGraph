use proc_macro::TokenStream;
// use proc_macro2;
use proc_macro_error::*;
use quote::ToTokens;
use syn::{parse_macro_input, Fields, Item, ItemStruct};

mod node_enum;
use node_enum::*;

mod typed_node;
use typed_node::*;

mod bidirectional;
use bidirectional::*;

mod utils;

// mod index_enum;
// use index_enum::*;

#[proc_macro]
#[proc_macro_error]
pub fn node_enum(macro_input: TokenStream) -> TokenStream {
  let macro_input: syn::File = parse_macro_input!(macro_input);
  let Item::Enum(the_enum) = &macro_input.items[0] else {
    abort!(macro_input.items[0], "The node enum should be the first item of node_enum!")
  };
  let enumt = the_enum.ident.clone();
  let vis = the_enum.vis.clone();

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

  if check_type_distinct(&vars) {
    for (ident, ty) in &vars {
      make_query_by_type_trait_impl(&mut result, &enumt, ident, ty, &vis);
      make_transaction_by_type_trait_impl(&mut result, &enumt, ident, ty);
    }
  }
  let source_enum_name = make_source_enum(&mut result, &vars, &enumt, &vis);
  let link_mirror_enum_name = make_link_mirror_enum(&mut result, &vars, &enumt, &vis);
  let node_type_mirror_name =
    make_node_type_mirror_enum(&mut result, &vars, &enumt, &vis);

  let mut bidirectional_links = Vec::new();
  for item in macro_input.items.iter().skip(1) {
    if let Item::Macro(the_macro) = item {
      if the_macro.mac.path.is_ident("bidirectional") {
        if get_bidiretional(the_macro.mac.tokens.clone(), &mut bidirectional_links)
          .is_err()
        {
          abort!(the_macro, "Parse failed")
        }
      } else {
        abort!(the_macro.mac.path, "Unsupported macro");
      }
    } else {
      abort!(item, "Unsupported item type");
    }
  }

  make_node_enum(
    &mut result,
    &vars,
    &enumt,
    &source_enum_name,
    &link_mirror_enum_name,
    &node_type_mirror_name,
    &bidirectional_links,
  );

  result.into()
}

#[proc_macro_derive(TypedNode)]
#[proc_macro_error]
pub fn typed_node(input: TokenStream) -> TokenStream {
  let input: ItemStruct = parse_macro_input!(input);
  let name = input.ident.clone();
  let vis = input.vis.clone();
  let generics = input.generics.clone();
  let (sources, data) = get_members(&input);
  let mut result = proc_macro2::TokenStream::new();

  let source_enum = make_node_source_enum(&mut result, &sources, &name, &vis);
  let link_mirror = make_link_mirror(&mut result, &sources, &name, &vis);
  make_typed_node(
    &mut result,
    &sources,
    &data,
    &name,
    &vis,
    &generics,
    &source_enum,
    &link_mirror,
  );

  result.into()
}

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
