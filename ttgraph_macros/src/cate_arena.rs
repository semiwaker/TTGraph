use super::*;

use proc_macro2::{self, TokenStream};
use quote::{format_ident, quote};
use syn::Visibility;

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
      let next = &vars[i + 1].0;
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

pub(crate) fn make_cate_arena_iter(
  generated: &mut TokenStream, vars: &Vec<(Ident, Type)>, enumt: &Ident, discriminant: &Ident, vis: &Visibility,
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
        Some((ttgraph::NodeIndex(*k), v))
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
      type Item = (ttgraph::NodeIndex, &'a #enumt);
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
    #[automatically_derived]
    impl<'a> ttgraph::NodeIter<'a , #enumt> for #iter_name<'a> {}
  }
  .to_tokens(generated);

  iter_name
}

pub(crate) fn make_cate_arena_iter_mut(
  result: &mut TokenStream, vars: &Vec<(Ident, Type)>, enumt: &Ident, discriminant: &Ident, vis: &Visibility,
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
        Some((ttgraph::NodeIndex(*k), v))
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
      type Item = (ttgraph::NodeIndex, &'a mut #enumt);
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
    #[automatically_derived]
    impl<'a> ttgraph::NodeIterMut<'a, #enumt> for #iter_name<'a> {}
  }
  .to_tokens(result);

  iter_name
}

pub(crate) fn make_cate_arena_intoiter(
  result: &mut TokenStream, vars: &Vec<(Ident, Type)>, enumt: &Ident, discriminant: &Ident, vis: &Visibility,
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
        Some((ttgraph::NodeIndex(k), v))
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
      type Item = (ttgraph::NodeIndex, #enumt);
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
    #[automatically_derived]
    impl ttgraph::NodeIntoIter<#enumt> for #iter_name {}
  }
  .to_tokens(result);

  iter_name
}

pub(crate) fn make_cate_arena(
  result: &mut TokenStream, vars: &Vec<(Ident, Type)>, enumt: &Ident, discriminant: &Ident, vis: &Visibility,
) -> Ident {
  let arena_name = format_ident!("{}Arena", enumt);

  let containers = Vec::from_iter(vars.iter().map(|(ident, _)| snake_case(ident)));

  let iter = make_cate_arena_iter(result, vars, enumt, discriminant, vis);
  let itermut = make_cate_arena_iter_mut(result, vars, enumt, discriminant, vis);
  let intoiter = make_cate_arena_intoiter(result, vars, enumt, discriminant, vis);

  let mut fields = Vec::new();
  for cont in containers.iter() {
    fields.push(quote! {#cont: ttgraph::ordermap::OrderMap<usize, #enumt>});
  }

  let mut fields_new = Vec::new();
  for cont in &containers {
    fields_new.push(quote! {#cont: ttgraph::ordermap::OrderMap::new()});
  }

  let mut get_container_arms = Vec::new();
  for (cont, (ident, _)) in containers.iter().zip(vars.iter()) {
    get_container_arms.push(quote! {Self::D::#ident => &self.#cont});
  }
  let mut get_container_mut_arms = Vec::new();
  for (cont, (ident, _)) in containers.iter().zip(vars.iter()) {
    get_container_mut_arms.push(quote! {Self::D::#ident => &mut self.#cont});
  }

  let mut merge_arms = Vec::new();
  for cont in &containers {
    merge_arms.push(quote! {self.#cont.append(&mut other.#cont);});
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

  // let mut contains = Vec::new();
  // for (ident, ty) in vars {
  //   contains.push(quote! {
  //     #[automatically_derived]
  //     impl ttgraph::Contains<#ty> for #enumt {
  //       fn unwrap(self) -> #ty { if let #enumt::#ident(x) = self {x} else {panic!("Unwrap failed")} }
  //       fn expect(self, msg: &str) -> #ty { if let #enumt::#ident(x) = self {x} else {panic!("{msg}")} }
  //     }
  //   });
  // }

  // let mut arena_contains = Vec::new();
  // for (cont, (_, ty)) in containers.iter().zip(vars.iter()) {
  //   arena_contains.push(quote! {
  //     #[automatically_derived]
  //     impl ttgraph::ArenaContains<#enumt, #ty> for #arena_name {
  //       fn get_container<'a>(&'a self) -> &'a ttgraph::ordermap::OrderMap<usize, #enumt> { &self.#cont }
  //       fn get_container_mut<'a>(&'a mut self) -> &'a mut ttgraph::ordermap::OrderMap<usize, #enumt> { &mut self.#cont }
  //     }
  //   });
  // }

  quote! {
    #vis struct #arena_name {
      _id_distributer: ttgraph::id_distributer::IdDistributer,
      _dispatcher: ttgraph::ordermap::OrderMap<ttgraph::NodeIndex, #discriminant>,
      #(#fields,)*
    }

    impl ttgraph::CateArena for #arena_name {
      type V = #enumt;
      type D = #discriminant;
      type Iter<'a> = #iter<'a>;
      type IterMut<'a> = #itermut<'a>;
      type IntoIter = #intoiter;

      fn new(id_distributer: ttgraph::id_distributer::IdDistributer) -> Self {
        Self{ _id_distributer: id_distributer, _dispatcher: ttgraph::ordermap::OrderMap::new(), #(#fields_new),* }
      }
      fn new_from_iter(id_distributer: ttgraph::id_distributer::IdDistributer, iter: impl std::iter::IntoIterator<Item=(ttgraph::NodeIndex, Self::V)>) -> Self {
        let mut result = Self::new(id_distributer);
        for (idx, node) in iter {
          let d = ttgraph::Discriminated::discriminant(&node);
          if result._dispatcher.insert(idx, d).is_some() {
            panic!("Duplicated index");
          }
          if result.get_container_mut(d).insert(idx.0, node).is_some() {
            panic!("Duplicated index");
          }
        }
        result
      }
      fn dispatch(&self, i: NodeIndex) -> Option<Self::D> {
        self._dispatcher.get(&i).map(|x|*x)
      }
      fn get_container<'a>(&'a self, d: Self::D) -> &'a ttgraph::ordermap::OrderMap<usize, Self::V> {
        match d { #(#get_container_arms),* }
      }
      fn get_container_mut<'a>(&'a mut self, d: Self::D) -> &'a mut ttgraph::ordermap::OrderMap<usize, Self::V> {
        match d { #(#get_container_mut_arms),* }
      }
      fn alloc(&mut self, d: Self::D) -> NodeIndex {
        let idx = NodeIndex(self._id_distributer.alloc());
        if self._dispatcher.insert(idx, d).is_some(){
          panic!("Duplicated allocation");
        }
        idx
      }
      fn alloc_untyped(&mut self) -> NodeIndex {
        NodeIndex(self._id_distributer.alloc())
      }
      fn fill_back_untyped(&mut self, i: NodeIndex, item: Self::V) {
        let d = ttgraph::Discriminated::discriminant(&item);
        self._dispatcher.insert(i, d);
        if self.get_container_mut(d).insert(i.0, item).is_some() {
          panic!("Fillback an occupied index {:?}", i);
        }
      }
      fn remove(&mut self, i: NodeIndex) -> Option<Self::V> {
        let d = self._dispatcher.swap_remove(&i);
        d.and_then(|d| self.get_container_mut(d).swap_remove(&i.0))
      }
      fn merge(&mut self, mut other: Self) where Self: Sized{
        self._dispatcher.append(&mut other._dispatcher);
        #(#merge_arms)*
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

    // impl std::iter::IntoIterator for #arena_name {
    //   type IntoIter = #intoiter;
    //   type Item = (ttgraph::NodeIndex, #enumt);
    //   fn into_iter(self) -> Self::IntoIter{ self.into_iter() }
    // }
    // impl<'a> std::iter::IntoIterator for &'a #arena_name {
    //   type IntoIter = #iter;
    //   type Item = (ttgraph::NodeIndex, &'a #enumt);
    //   fn into_iter(self) -> Self::IntoIter{ self.iter() }
    // }
    // impl<'a> std::iter::IntoIterator for &'a mut #arena_name {
    //   type IntoIter = #iter;
    //   type Item = (ttgraph::NodeIndex, &'a mut #enumt);
    //   fn into_iter(self) -> Self::IntoIter{ self.iter_mut() }
    // }
  }
  .to_tokens(result);

  arena_name
}
