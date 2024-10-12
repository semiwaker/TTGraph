use super::*;

use proc_macro2::{self, TokenStream};
use quote::{format_ident, quote};
use syn::{Generics, Visibility};

pub(crate) fn make_node_discriminant(
  result: &mut TokenStream, vars: &Vec<(Ident, Type)>, enumt: &Ident, vis: &Visibility,
) -> Ident {
  let enum_name = format_ident!("{}Discriminant", enumt);
  let mut v = Vec::new();
  for (ident, _) in vars {
    v.push(quote! {#ident,});
  }

  let mut next_arms = Vec::new();
  for i in 0..vars.len() {
    let cur = &vars[i].0;
    next_arms.push(if i + 1 < vars.len() {
      let next = &vars[i].0;
      quote! {Self::#cur => Some(Self::#next)}
    } else {
      quote! {Self::#cur => None}
    })
  }

  let first = &vars.first().unwrap().0;

  quote! {
    #[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, std::hash::Hash)]
    #vis enum #enum_name{
      #(#v)*
    }
    #[automatically_derived]
    impl ttgraph::NodeDiscriminant for #enum_name {
      fn first() -> Self { Self::#first }
      fn next(&self) -> Option<Self> {
        match self {
          #(#next_arms),*
        }
      }
    }
  }
  .to_tokens(result);

  enum_name
}

pub(crate) fn make_node_index(
  result: &mut TokenStream, vars: &Vec<(Ident, Type)>, enumt: &Ident, discriminant: &Ident, vis: &Visibility,
) -> Ident {
  let node_index_name = format_ident!("{}NodeIndex", enumt);

  let mut fields = Vec::new();
  for (ident, _) in vars {
    fields.push(quote! {#ident(usize)});
  }
  fields.push(quote! {TTGraphEmptyIndex});

  let mut disc_arms = Vec::new();
  for (ident, _) in vars {
    disc_arms.push(quote! { Self::#ident(_) => #discriminant::#ident })
  }
  disc_arms.push(quote! {Self::TTGraphEmptyIndex => panic!("Empty index have no discriminant: {:?}", self)});

  let mut id_arms = Vec::new();
  for (ident, _) in vars {
    id_arms.push(quote! { Self::#ident(x) => *x })
  }
  id_arms.push(quote! {Self::TTGraphEmptyIndex => 0});

  let mut idx_contains = Vec::new();
  for (ident, ty) in vars {
    idx_contains.push(quote! {
      #[automatically_derived]
      impl ttgraph::IdxContains<#ty> for #node_index_name {
        fn wrap(id: usize) -> Self { Self::#ident(id) }
      }
    });
  }

  quote! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, std::hash::Hash)]
    #vis enum #node_index_name {
      #(#fields),*
    }

    #[automatically_derived]
    impl ttgraph::CateIndex for #node_index_name {
      type D = #discriminant;
      type Data = #enumt;

      fn empty() -> Self { Self::TTGraphEmptyIndex }
      fn is_empty(&self) -> bool { if let Self::TTGraphEmptyIndex = self {true} else {false} }
      fn id(&self) -> usize {
        match self { #(#id_arms),* }
      }
    }

    #[automatically_derived]
    impl ttgraph::Discriminated<#discriminant> for #node_index_name {
      fn discriminant(&self) -> #discriminant {
        match self { #(#disc_arms),* }
      }
    }

    #(#idx_contains)*
  }
  .to_tokens(result);

  node_index_name
}

pub(crate) fn make_cate_arena_iter(
  generated: &mut TokenStream, vars: &Vec<(Ident, Type)>, enumt: &Ident, node_index: &Ident, discriminant: &Ident,
  vis: &Visibility,
) -> Ident {
  let iter_name = format_ident!("{}Iter", enumt);

  let iterators = Vec::from_iter(vars.iter().map(|(ident, _)| snake_case(ident)));
  let mut fields = Vec::new();
  for iter in iterators.iter() {
    fields.push(quote! {#iter: ttgraph::ordermap::map::Iter<'a, usize, #enumt>});
  }

  let mut next_arms = Vec::new();
  for (iter, (ident, _)) in iterators.iter().zip(vars.iter()) {
    next_arms.push(quote! {Some(#discriminant::#ident) => {
      if let Some((k, v)) = std::iter::Iterator::next(&mut self.#iter) {
        Some((#node_index::#ident(*k), v))
      } else {
        self._iter_state = ttgraph::NodeDiscriminant::next(&#discriminant::#ident);
        self.next()
      }
    }});
  }

  let mut sizes = Vec::new();
  for iter in iterators.iter() {
    sizes.push(quote! {std::iter::Iterator::size_hint(&self.#iter)})
  }

  quote! {
    #[derive(Clone)]
    #vis struct #iter_name<'a> {
      _iter_state: Option<#discriminant>,
      #(#fields),*
    }

    #[automatically_derived]
    impl<'a> std::iter::Iterator for #iter_name<'a> {
      type Item = (#node_index, &'a #enumt);
      fn next(&mut self) -> Option<Self::Item> {
        match self._iter_state {
          #(#next_arms),*
          None => None,
        }
      }
      fn size_hint(&self) -> (usize, Option<usize>) {
        [#(#sizes),*].into_iter().reduce(|a, b|(a.0+b.0, a.1.and_then(|x|b.1.and_then(|y|Some(x+y))))).unwrap_or((0, Some(0)))
      }
    }

    #[automatically_derived]
    impl<'a> std::iter::ExactSizeIterator for #iter_name<'a> {}
    #[automatically_derived]
    impl<'a> std::iter::FusedIterator for #iter_name<'a> {}
  }
  .to_tokens(generated);

  iter_name
}

pub(crate) fn make_cate_arena_iter_mut(
  result: &mut TokenStream, vars: &Vec<(Ident, Type)>, enumt: &Ident, node_index: &Ident, discriminant: &Ident,
  vis: &Visibility,
) -> Ident {
  let iter_name = format_ident!("{}IterMut", enumt);

  let iterators = Vec::from_iter(vars.iter().map(|(ident, _)| snake_case(ident)));
  let mut fields = Vec::new();
  for iter in iterators.iter() {
    fields.push(quote! {#iter: ttgraph::ordermap::map::IterMut<'a, usize, #enumt>});
  }

  let mut next_arms = Vec::new();
  for (iter, (ident, _)) in iterators.iter().zip(vars.iter()) {
    next_arms.push(quote! {Some(#discriminant::#ident) => {
      if let Some((k, v)) = std::iter::Iterator::next(&mut self.#iter) {
        Some((#node_index::#ident(*k), v))
      } else {
        self._iter_state = ttgraph::NodeDiscriminant::next(&#discriminant::#ident);
        self.next()
      }
    }});
  }

  let mut sizes = Vec::new();
  for iter in iterators.iter() {
    sizes.push(quote! {std::iter::Iterator::size_hint(&self.#iter)})
  }

  quote! {
    #vis struct #iter_name<'a> {
      _iter_state: Option<#discriminant>,
      #(#fields),*
    }

    #[automatically_derived]
    impl<'a> std::iter::Iterator for #iter_name<'a> {
      type Item = (#node_index, &'a mut #enumt);
      fn next(&mut self) -> Option<Self::Item> {
        match self._iter_state {
          #(#next_arms),*
          None => None,
        }
      }
      fn size_hint(&self) -> (usize, Option<usize>) {
        [#(#sizes),*].into_iter().reduce(|a, b|(a.0+b.0, a.1.and_then(|x|b.1.and_then(|y|Some(x+y))))).unwrap_or((0, Some(0)))
      }
    }

    #[automatically_derived]
    impl<'a> std::iter::ExactSizeIterator for #iter_name<'a> {}
    #[automatically_derived]
    impl<'a> std::iter::FusedIterator for #iter_name<'a> {}
  }
  .to_tokens(result);

  iter_name
}

pub(crate) fn make_cate_arena_intoiter(
  result: &mut TokenStream, vars: &Vec<(Ident, Type)>, enumt: &Ident, node_index: &Ident, discriminant: &Ident,
  vis: &Visibility,
) -> Ident {
  let iter_name = format_ident!("{}IntoIter", enumt);

  let iterators = Vec::from_iter(vars.iter().map(|(ident, _)| snake_case(ident)));
  let mut fields = Vec::new();
  for iter in iterators.iter() {
    fields.push(quote! {#iter: ttgraph::ordermap::map::IntoIter<usize, #enumt>});
  }

  let mut next_arms = Vec::new();
  for (iter, (ident, _)) in iterators.iter().zip(vars.iter()) {
    next_arms.push(quote! {Some(#discriminant::#ident) => {
      if let Some((k, v)) = std::iter::Iterator::next(&mut self.#iter) {
        Some((#node_index::#ident(k), v))
      } else {
        self._iter_state = ttgraph::NodeDiscriminant::next(&#discriminant::#ident);
        self.next()
      }
    }});
  }

  let mut sizes = Vec::new();
  for iter in iterators.iter() {
    sizes.push(quote! {std::iter::Iterator::size_hint(&self.#iter)})
  }

  quote! {
    #vis struct #iter_name {
      _iter_state: Option<#discriminant>,
      #(#fields),*
    }

    #[automatically_derived]
    impl std::iter::Iterator for #iter_name {
      type Item = (#node_index, #enumt);
      fn next(&mut self) -> Option<Self::Item> {
        match self._iter_state {
          #(#next_arms),*
          None => None,
        }
      }
      fn size_hint(&self) -> (usize, Option<usize>) {
        [#(#sizes),*].into_iter().reduce(|a, b|(a.0+b.0, a.1.and_then(|x|b.1.and_then(|y|Some(x+y))))).unwrap_or((0, Some(0)))
      }
    }

    #[automatically_derived]
    impl std::iter::ExactSizeIterator for #iter_name {}
    #[automatically_derived]
    impl std::iter::FusedIterator for #iter_name {}
  }
  .to_tokens(result);

  iter_name
}

pub(crate) fn make_cate_arena(
  result: &mut TokenStream, vars: &Vec<(Ident, Type)>, enumt: &Ident, discriminant: &Ident, node_index: &Ident,
  vis: &Visibility,
) -> Ident {
  let arena_name = format_ident!("{}Arena", enumt);

  let containers = Vec::from_iter(vars.iter().map(|(ident, _)| snake_case(ident)));

  let iter = make_cate_arena_iter(result, vars, enumt, node_index, discriminant, vis);
  let itermut = make_cate_arena_iter_mut(result, vars, enumt, node_index, discriminant, vis);
  let intoiter = make_cate_arena_intoiter(result, vars, enumt, node_index, discriminant, vis);

  let mut fields = Vec::new();
  for cont in containers.iter() {
    fields.push(quote! {#cont: ttgraph::ordermap::OrderMap<usize, #enumt>});
  }

  let mut fields_new = Vec::new();
  for ident in &containers {
    fields_new.push(quote! {#ident: ttgraph::ordermap::OrderMap::new()});
  }

  let mut insert_arms = Vec::new();
  for (cont, (ident, _)) in containers.iter().zip(vars.iter()) {
    insert_arms.push(quote! {Self::V::#ident(_) => {
      let idx = self.alloc_id();
      self.#cont.insert(idx, item);
      Self::K::#ident(idx)
    }});
  }

  let mut alloc_arms = Vec::new();
  for (ident, _) in vars.iter() {
    alloc_arms.push(quote! {Self::D::#ident => Self::K::#ident(self.alloc_id()) });
  }

  let mut fill_back_arms = Vec::new();
  for (cont, (ident, _)) in containers.iter().zip(vars.iter()) {
    fill_back_arms.push(quote! {Self::K::#ident(idx) =>
      if let Self::V::#ident(_) = item{
        self.#cont.insert(idx, item);
      } else { panic!("Fillback {:?} with incompatible type {:?}", i, ttgraph::Discriminated::<Self::D>::discriminant(&item)) }
    });
  }

  let mut remove_arms = Vec::new();
  for (cont, (ident, _)) in containers.iter().zip(vars.iter()) {
    remove_arms.push(quote! {Self::K::#ident(idx) => self.#cont.swap_remove(&idx) });
  }

  let mut contains_arms = Vec::new();
  for (cont, (ident, _)) in containers.iter().zip(vars.iter()) {
    contains_arms.push(quote! {Self::K::#ident(idx) => self.#cont.contains_key(&idx) });
  }

  let mut get_arms = Vec::new();
  for (cont, (ident, _)) in containers.iter().zip(vars.iter()) {
    get_arms.push(quote! {Self::K::#ident(idx) => self.#cont.get(&idx) });
  }

  let mut get_mut_arms = Vec::new();
  for (cont, (ident, _)) in containers.iter().zip(vars.iter()) {
    get_mut_arms.push(quote! {Self::K::#ident(idx) => self.#cont.get_mut(&idx) });
  }

  let mut lens = Vec::new();
  for cont in containers.iter() {
    lens.push(quote! { self.#cont.len() });
  }

  let mut iter_arms = Vec::new();
  for cont in containers.iter() {
    iter_arms.push(quote! { #cont: self.#cont.iter() });
  }

  let mut iter_mut_arms = Vec::new();
  for cont in containers.iter() {
    iter_mut_arms.push(quote! { #cont: self.#cont.iter_mut() });
  }

  let mut intoiter_arms = Vec::new();
  for cont in containers.iter() {
    intoiter_arms.push(quote! { #cont: self.#cont.into_iter() });
  }

  let mut contains = Vec::new();
  for (ident, ty) in vars {
    contains.push(quote! {
      #[automatically_derived]
      impl ttgraph::Contains<#ty> for #enumt {
        fn unwrap(self) -> #ty { if let #enumt::#ident(x) = self {x} else {panic!("Unwrap failed")} }
        fn expect(self, msg: &str) -> #ty { if let #enumt::#ident(x) = self {x} else {panic!("{msg}")} }
      }
    });
  }

  let mut arena_contains = Vec::new();
  for (cont, (_, ty)) in containers.iter().zip(vars.iter()) {
    arena_contains.push(quote! {
      #[automatically_derived]
      impl ttgraph::ArenaContains<#enumt, #ty> for #arena_name {
        fn get_container<'a>(&'a self) -> &'a ttgraph::ordermap::OrderMap<usize, #enumt> { &self.#cont }
        fn get_container_mut<'a>(&'a mut self) -> &'a mut ttgraph::ordermap::OrderMap<usize, #enumt> { &mut self.#cont }
      }
    });
  }

  quote! {
    #vis struct #arena_name {
      distributer: ttgraph::id_distributer::IdDistributer,
      #(#fields,)*
    }
    #(#contains)*
    #(#arena_contains)*

    impl ttgraph::CateArena for #arena_name {
      type K = #node_index;
      type V = #enumt;
      type D = #discriminant;
      type Iter<'a> = #iter<'a>;
      type IterMut<'a> = #itermut<'a>;
      type IntoIter = #intoiter;

      fn new(distributer: ttgraph::id_distributer::IdDistributer) -> Self {
        Self{ distributer, #(#fields_new),* }
      }
      fn get_distributor(&self) -> &ttgraph::id_distributer::IdDistributer {&self.distributer}
      fn insert(&mut self, item: Self::V) -> #node_index {
        match item { #(#insert_arms),* }
      }
      fn alloc(&mut self, ty: Self::D) -> Self::K {
        match ty { #(#alloc_arms),* }
      }
      fn fill_back(&mut self, i: Self::K, item: Self::V) {
        match i { Self::K::TTGraphEmptyIndex => panic!("Fillback an empty index"), #(#fill_back_arms),*  }
      }
      fn remove(&mut self, i: Self::K) -> Option<Self::V> {
        match i { Self::K::TTGraphEmptyIndex => panic!("Remove an empty index"), #(#remove_arms),* }
      }
      fn contains(&self, i: Self::K) -> bool {
        match i { Self::K::TTGraphEmptyIndex => panic!("Contains an empty index"), #(#contains_arms),* }
      }
      fn get(&self, i: Self::K) -> Option<&Self::V> {
        match i { Self::K::TTGraphEmptyIndex => panic!("Get an empty index"), #(#get_arms),* }
      }
      fn get_mut(&mut self, i: Self::K) -> Option<&mut Self::V> {
        match i { Self::K::TTGraphEmptyIndex => panic!("Getmut an empty index"), #(#get_mut_arms),* }
      }
      fn len(&self) -> usize {
        #(#lens)+*
      }
      fn iter<'a>(&'a self) -> Self::Iter<'a> {
        Self::Iter{ _iter_state: Some(<Self::D as ttgraph::NodeDiscriminant>::first()), #(#iter_arms),* }
      }
      fn iter_mut<'a>(&'a mut self) -> Self::IterMut<'a> {
        Self::IterMut{ _iter_state: Some(<Self::D as ttgraph::NodeDiscriminant>::first()), #(#iter_mut_arms),* }
      }
      fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter{ _iter_state: Some(<Self::D as ttgraph::NodeDiscriminant>::first()), #(#intoiter_arms),* }
      }
    }
  }
  .to_tokens(result);

  arena_name
}
