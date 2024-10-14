use std::fmt::Debug;
use std::{hash::Hash, iter::FusedIterator};

use crate::id_distributer::IdDistributer;
use crate::{NodeEnum, NodeIndex};
use ordermap::OrderMap;

/// A discriminant enum for the CateIndex & NodeEnum
pub trait NodeDiscriminant:
  Debug + Clone + Copy + PartialEq + Eq + Hash + Ord + PartialOrd + Sized + Sync + Send + 'static
{
  /// Get the first discriminant of the NodeEnum
  fn first() -> Self;
  /// Get the next discriminant of the NodeEnum
  fn next(&self) -> Option<Self>;
}

pub trait Discriminated<D: NodeDiscriminant> {
  fn discriminant(&self) -> D;
}

// pub trait Contains<T>: NodeEnum {
//   fn wrap(id: usize) -> NodeIndex;
//   fn unwrap(self) -> T;
//   fn expect(self, msg: &str) -> T;
// }

// pub trait ArenaContains<V: Contains<T>, T> {
//   fn get_container(&self) -> &OrderMap<usize, V>;
//   fn get_container_mut(&mut self) -> &mut OrderMap<usize, V>;
// }

pub trait NodeIter<'a, T: NodeEnum + 'a>:
  Clone + ExactSizeIterator + FusedIterator + Iterator<Item = (NodeIndex, &'a T)> + Sized
{
}
pub trait NodeIterMut<'a, T: NodeEnum + 'a>:
  ExactSizeIterator + FusedIterator + Iterator<Item = (NodeIndex, &'a mut T)> + Sized
{
}
pub trait NodeIntoIter<T: NodeEnum>:
  ExactSizeIterator + FusedIterator + Iterator<Item = (NodeIndex, T)> + Sized
{
}

pub trait CateArena: 'static {
  // Required
  type V: NodeEnum<Discriminant = Self::D>;
  type D: NodeDiscriminant;
  type Iter<'a>: NodeIter<'a, Self::V>;
  type IterMut<'a>: NodeIterMut<'a, Self::V>;
  type IntoIter: NodeIntoIter<Self::V>;

  fn new(id_distributer: IdDistributer) -> Self;
  fn new_from_iter(id_distributer: IdDistributer, iter: impl IntoIterator<Item = (NodeIndex, Self::V)>) -> Self;
  fn dispatch(&self, i: NodeIndex) -> Option<Self::D>;
  fn get_container<'a>(&'a self, d: Self::D) -> &'a OrderMap<usize, Self::V>;
  fn get_container_mut<'a>(&'a mut self, d: Self::D) -> &'a mut OrderMap<usize, Self::V>;
  fn alloc(&mut self, d: Self::D) -> NodeIndex;
  fn alloc_untyped(&mut self) -> NodeIndex;
  fn fill_back_untyped(&mut self, i: NodeIndex, item: Self::V);
  fn merge(&mut self, other: Self)
  where
    Self: Sized;
  fn remove(&mut self, i: NodeIndex) -> Option<Self::V>;
  fn len(&self) -> usize;
  fn iter<'a>(&'a self) -> Self::Iter<'a>;
  fn iter_mut<'a>(&'a mut self) -> Self::IterMut<'a>;
  fn into_iter(self) -> Self::IntoIter;

  // Provided
  fn insert(&mut self, item: Self::V) -> NodeIndex {
    let d = Discriminated::discriminant(&item);
    let idx = self.alloc(d);
    if self.get_container_mut(d).insert(idx.0, item).is_some() {
      panic!("Allocated index already existed, check if the context is correct!");
    }
    idx
  }
  fn fill_back(&mut self, i: NodeIndex, item: Self::V) {
    let d = self.dispatch(i).expect(&format!("Fillback an non-existing index {:?}", i));
    if Discriminated::discriminant(&item) != d {
      panic!("Fillback with incompatible type: found{:?} expect{:?}", Discriminated::discriminant(&item), d);
    }
    if self.get_container_mut(d).insert(i.0, item).is_some() {
      panic!("Fillback an occupied index {:?}", i);
    }
  }
  fn contains(&self, i: NodeIndex) -> bool {
    self.dispatch(i).map_or(false, |d| self.get_container(d).contains_key(&i.0))
  }
  fn get(&self, i: NodeIndex) -> Option<&Self::V> {
    self.dispatch(i).and_then(|d| self.get_container(d).get(&i.0))
  }
  fn get_mut(&mut self, i: NodeIndex) -> Option<&mut Self::V> {
    self.dispatch(i).and_then(|d| self.get_container_mut(d).get_mut(&i.0))
  }
  fn update_with<F>(&mut self, i: NodeIndex, f: F)
  where
    F: FnOnce(Self::V) -> Self::V,
  {
    let d = self.dispatch(i).expect(&format!("Update a non-existing node {:?}", i));
    let x = self.get_container_mut(d).swap_remove(&i.0).expect(&format!("Update a non-existing node {:?}", i));
    self.get_container_mut(d).insert(i.0, f(x));
  }

  // fn iter_cate<'a, T: 'a>(&'a self) -> IterCate<'a, Self::K, Self::V, T>
  // where
  //   Self::K: IdxContains<T>,
  //   Self::V: Contains<T>,
  //   &'a Self::V: Contains<&'a T>,
  //   Self: ArenaContains<Self::V, T>,
  // {
  //   IterCate(self.get_container().iter(), PhantomData, PhantomData)
  // }
  // fn iter_mut_cate<'a, T: 'a>(&'a mut self) -> IterCateMut<'a, Self::K, Self::V, T>
  // where
  //   Self::K: IdxContains<T>,
  //   Self::V: Contains<T>,
  //   &'a mut Self::V: Contains<&'a mut T>,
  //   Self: ArenaContains<Self::V, T>,
  // {
  //   IterCateMut(self.get_container_mut().iter_mut(), PhantomData, PhantomData)
  // }
}

// #[derive(Clone, Default)]
// pub struct IterCate<'a, K, V, T>(map::Iter<'a, usize, V>, PhantomData<K>, PhantomData<T>)
// where
//   K: CateIndex<Data = V> + IdxContains<T>,
//   V: CateNode<Index = K> + Contains<T> + 'static,
//   &'a V: Contains<&'a T>,
//   T: 'a;

// impl<'a, K, V, T> Iterator for IterCate<'a, K, V, T>
// where
//   K: CateIndex<Data = V> + IdxContains<T>,
//   V: CateNode<Index = K> + Contains<T> + 'static,
//   &'a V: Contains<&'a T>,
//   T: 'a,
// {
//   type Item = (K, &'a T);
//   fn next(&mut self) -> Option<Self::Item> {
//     self.0.next().and_then(|(id, data)| Some((<K as IdxContains<T>>::wrap(*id), data.unwrap())))
//   }
//   fn size_hint(&self) -> (usize, Option<usize>) {
//     self.0.size_hint()
//   }
// }

// impl<'a, K, V, T> FusedIterator for IterCate<'a, K, V, T>
// where
//   K: CateIndex<Data = V> + IdxContains<T>,
//   V: CateNode<Index = K> + Contains<T> + 'static,
//   &'a V: Contains<&'a T>,
//   T: 'a,
// {
// }

// impl<'a, K, V, T> ExactSizeIterator for IterCate<'a, K, V, T>
// where
//   K: CateIndex<Data = V> + IdxContains<T>,
//   V: CateNode<Index = K> + Contains<T> + 'static,
//   &'a V: Contains<&'a T>,
//   T: 'a,
// {
// }

// #[derive(Default)]
// pub struct IterCateMut<'a, K, V, T>(map::IterMut<'a, usize, V>, PhantomData<K>, PhantomData<T>)
// where
//   K: CateIndex<Data = V> + IdxContains<T>,
//   V: CateNode<Index = K> + Contains<T> + 'static,
//   &'a mut V: Contains<&'a mut T>,
//   T: 'a;

// impl<'a, K, V, T> Iterator for IterCateMut<'a, K, V, T>
// where
//   K: CateIndex<Data = V> + IdxContains<T>,
//   V: CateNode<Index = K> + Contains<T> + 'static,
//   &'a mut V: Contains<&'a mut T>,
//   T: 'a,
// {
//   type Item = (K, &'a mut T);
//   fn next(&mut self) -> Option<Self::Item> {
//     self.0.next().and_then(|(id, data)| Some((<K as IdxContains<T>>::wrap(*id), data.unwrap())))
//   }
//   fn size_hint(&self) -> (usize, Option<usize>) {
//     self.0.size_hint()
//   }
// }

// impl<'a, K, V, T> FusedIterator for IterCateMut<'a, K, V, T>
// where
//   K: CateIndex<Data = V> + IdxContains<T>,
//   V: CateNode<Index = K> + Contains<T> + 'static,
//   &'a mut V: Contains<&'a mut T>,
//   T: 'a,
// {
// }

// impl<'a, K, V, T> ExactSizeIterator for IterCateMut<'a, K, V, T>
// where
//   K: CateIndex<Data = V> + IdxContains<T>,
//   V: CateNode<Index = K> + Contains<T> + 'static,
//   &'a mut V: Contains<&'a mut T>,
//   T: 'a,
// {
// }
