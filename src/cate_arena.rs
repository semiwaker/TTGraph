use std::fmt::Debug;
use std::marker::PhantomData;
use std::{hash::Hash, iter::FusedIterator};

use crate::id_distributer::IdDistributer;
use ordermap::{map, OrderMap};

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

/// A index for the NodeEnum
pub trait CateIndex:
  Debug + Clone + Copy + PartialEq + Eq + Hash + Ord + PartialOrd + Sized + 'static + Discriminated<Self::D>
{
  type D: NodeDiscriminant;
  type Data: CateNode<D = Self::D, Index = Self>;
  /// Make an index pointing empty
  fn empty() -> Self;
  /// Check if the index points to empty
  fn is_empty(&self) -> bool;
  /// Return a raw id of the node
  fn id(&self) -> usize;
}

pub trait CateNode: 'static + Discriminated<Self::D> {
  type D: NodeDiscriminant;
  type Index: CateIndex<D = Self::D, Data = Self>;
  fn empty_index() -> Self::Index {
    Self::Index::empty()
  }
}

pub trait IdxContains<T>: CateIndex {
  fn wrap(id: usize) -> Self;
}
pub trait Contains<T> {
  fn unwrap(self) -> T;
  fn expect(self, msg: &str) -> T;
}

pub trait ArenaContains<V: Contains<T>, T> {
  fn get_container(&self) -> &OrderMap<usize, V>;
  fn get_container_mut(&mut self) -> &mut OrderMap<usize, V>;
}

pub trait CateArena {
  // Required
  type K: CateIndex<Data = Self::V, D = Self::D>;
  type V: CateNode<Index = Self::K, D = Self::D>;
  type D: NodeDiscriminant;
  type Iter<'a>: Clone + ExactSizeIterator + FusedIterator + Iterator<Item = (Self::K, &'a Self::V)> + Sized
  where
    Self: 'a;
  type IterMut<'a>: ExactSizeIterator + FusedIterator + Iterator<Item = (Self::K, &'a mut Self::V)> + Sized
  where
    Self: 'a;
  type IntoIter: ExactSizeIterator + FusedIterator + Iterator<Item = (Self::K, Self::V)> + Sized;

  fn new(distributer: IdDistributer) -> Self;
  fn get_distributor(&self) -> &IdDistributer;

  fn insert(&mut self, item: Self::V) -> Self::K;
  fn alloc(&mut self, ty: Self::D) -> Self::K;
  fn fill_back(&mut self, i: Self::K, item: Self::V);
  fn remove(&mut self, i: Self::K) -> Option<Self::V>;
  fn contains(&self, i: Self::K) -> bool;
  fn get(&self, i: Self::K) -> Option<&Self::V>;
  fn get_mut(&mut self, i: Self::K) -> Option<&mut Self::V>;
  fn len(&self) -> usize;

  fn iter<'a>(&'a self) -> Self::Iter<'a>;
  fn iter_mut<'a>(&'a mut self) -> Self::IterMut<'a>;
  fn into_iter(self) -> Self::IntoIter;

  fn update_with<F>(&mut self, i: Self::K, f: F)
  where
    F: FnOnce(Self::V) -> Self::V,
  {
    let x = self.remove(i).expect(&format!("Update a non-existing node {:?}", i));
    self.insert(f(x));
  }

  // Provided
  fn iter_cate<'a, T: 'a>(&'a self) -> IterCate<'a, Self::K, Self::V, T>
  where
    Self::K: IdxContains<T>,
    Self::V: Contains<T>,
    &'a Self::V: Contains<&'a T>,
    Self: ArenaContains<Self::V, T>,
  {
    IterCate(self.get_container().iter(), PhantomData, PhantomData)
  }
  fn iter_mut_cate<'a, T: 'a>(&'a mut self) -> IterCateMut<'a, Self::K, Self::V, T>
  where
    Self::K: IdxContains<T>,
    Self::V: Contains<T>,
    &'a mut Self::V: Contains<&'a mut T>,
    Self: ArenaContains<Self::V, T>,
  {
    IterCateMut(self.get_container_mut().iter_mut(), PhantomData, PhantomData)
  }

  fn alloc_id(&self) -> usize {
    self.get_distributor().alloc()
  }
  fn merge(&mut self, other: Self)
  where
    Self: Sized,
  {
    for (k, v) in other.into_iter() {
      self.fill_back(k, v);
    }
  }
}

#[derive(Clone, Default)]
pub struct IterCate<'a, K, V, T>(map::Iter<'a, usize, V>, PhantomData<K>, PhantomData<T>)
where
  K: CateIndex<Data = V> + IdxContains<T>,
  V: CateNode<Index = K> + Contains<T> + 'static,
  &'a V: Contains<&'a T>,
  T: 'a;

impl<'a, K, V, T> Iterator for IterCate<'a, K, V, T>
where
  K: CateIndex<Data = V> + IdxContains<T>,
  V: CateNode<Index = K> + Contains<T> + 'static,
  &'a V: Contains<&'a T>,
  T: 'a,
{
  type Item = (K, &'a T);
  fn next(&mut self) -> Option<Self::Item> {
    self.0.next().and_then(|(id, data)| Some((<K as IdxContains<T>>::wrap(*id), data.unwrap())))
  }
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.0.size_hint()
  }
}

impl<'a, K, V, T> FusedIterator for IterCate<'a, K, V, T>
where
  K: CateIndex<Data = V> + IdxContains<T>,
  V: CateNode<Index = K> + Contains<T> + 'static,
  &'a V: Contains<&'a T>,
  T: 'a,
{
}

impl<'a, K, V, T> ExactSizeIterator for IterCate<'a, K, V, T>
where
  K: CateIndex<Data = V> + IdxContains<T>,
  V: CateNode<Index = K> + Contains<T> + 'static,
  &'a V: Contains<&'a T>,
  T: 'a,
{
}

#[derive(Default)]
pub struct IterCateMut<'a, K, V, T>(map::IterMut<'a, usize, V>, PhantomData<K>, PhantomData<T>)
where
  K: CateIndex<Data = V> + IdxContains<T>,
  V: CateNode<Index = K> + Contains<T> + 'static,
  &'a mut V: Contains<&'a mut T>,
  T: 'a;

impl<'a, K, V, T> Iterator for IterCateMut<'a, K, V, T>
where
  K: CateIndex<Data = V> + IdxContains<T>,
  V: CateNode<Index = K> + Contains<T> + 'static,
  &'a mut V: Contains<&'a mut T>,
  T: 'a,
{
  type Item = (K, &'a mut T);
  fn next(&mut self) -> Option<Self::Item> {
    self.0.next().and_then(|(id, data)| Some((<K as IdxContains<T>>::wrap(*id), data.unwrap())))
  }
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.0.size_hint()
  }
}

impl<'a, K, V, T> FusedIterator for IterCateMut<'a, K, V, T>
where
  K: CateIndex<Data = V> + IdxContains<T>,
  V: CateNode<Index = K> + Contains<T> + 'static,
  &'a mut V: Contains<&'a mut T>,
  T: 'a,
{
}

impl<'a, K, V, T> ExactSizeIterator for IterCateMut<'a, K, V, T>
where
  K: CateIndex<Data = V> + IdxContains<T>,
  V: CateNode<Index = K> + Contains<T> + 'static,
  &'a mut V: Contains<&'a mut T>,
  T: 'a,
{
}
