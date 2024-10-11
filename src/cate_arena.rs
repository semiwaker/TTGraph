use std::{hash::Hash, iter::FusedIterator};
use std::marker::PhantomData;
use std::fmt::Debug;

use crate::id_distributer::IdDistributer;
use ordermap::{map, OrderMap};

pub trait CateIndex: Debug + Clone + Copy + PartialEq + Eq + Hash + Ord + PartialOrd + Sized + 'static{
  type Discriminant: Debug + Clone + Copy + PartialEq + Eq + Hash + Ord + PartialOrd + Sized + 'static;
  type Data;
  fn empty() -> Self;
  fn is_empty(&self) -> bool;
  fn discriminant(&self) -> Self::Discriminant;
  fn id(&self) -> usize;
}

pub trait IndexedBy<E: Debug + Clone + Copy + PartialEq + Eq + Hash + Ord + PartialOrd + Sized + 'static> {
}

pub trait IdxContains<T: 'static> {
  fn wrap(id: usize) -> Self;
}
pub trait Contains<T: 'static> {
}
pub trait ArenaContains<T: 'static> {
  fn get_container(&self) -> &OrderMap<usize, T>;
  fn get_container_mut(&mut self) -> &mut OrderMap<usize, T>;
}

pub trait CateArena<K: CateIndex<Data=V>, V: IndexedBy<K> + 'static> {
  type Iter<'a>: Debug + Clone + ExactSizeIterator + FusedIterator + Iterator<Item=(K, &'a V)> + Sized where Self: 'a;
  type IterMut<'a>: Debug + Clone + ExactSizeIterator + FusedIterator + Iterator<Item=(K, &'a mut V)> + Sized where Self: 'a;
  type Drain: Debug + Clone + ExactSizeIterator + FusedIterator + Iterator<Item=(K, V)> + Sized;

  fn new(distributer: IdDistributer) -> Self;
  fn get_distributor(&self) -> &IdDistributer;

  fn insert(&mut self, item:V) -> K;
  fn alloc(&mut self, ty: K::Discriminant) -> K;
  fn fill_back(&mut self, i: K, item: V);
  fn remove(&mut self, i: K) -> Option<V>;
  fn contains(&self, i: K) -> bool;
  fn get(&self, i: K) -> Option<&V>;
  fn get_mut(&mut self, i: K) -> Option<&mut V>;
  fn update_with<F>(&mut self, i: K, f: F) where F: FnOnce(V) -> V;
  fn len(&self) -> usize;

  fn iter(&self) -> Self::Iter<'_>;
  fn iter_mut(&mut self) -> Self::IterMut<'_>;
  fn drain(self) -> Self::Drain;

  fn update_with_cate<F, T: 'static>(&mut self, i: K, f: F) where F: FnOnce(T) -> T, V: Contains<T>;

  fn iter_cate<T>(&self) -> IterCate<'_, K, V, T> where K: IdxContains<T>, V: Contains<T>, Self:ArenaContains<T> {
    IterCate(self.get_container().iter(), PhantomData, PhantomData)
  }
  fn iter_mut_cate<T>(&mut self) -> IterCateMut<'_, K, V, T> where K: IdxContains<T>, V: Contains<T>, Self:ArenaContains<T> {
    IterCateMut(self.get_container_mut().iter_mut(), PhantomData, PhantomData)
  }

  fn alloc_id(&self) -> usize {
    self.get_distributor().alloc()
  }
  fn merge(&mut self, other: Self) where Self: Sized {
    for (k, v) in other.drain() {
      self.fill_back(k, v);
    }
  }
}

#[derive(Debug, Clone, Default)]
pub struct IterCate<'a, K, V, T>(map::Iter<'a, usize, T>, PhantomData<K>, PhantomData<V>) where 
  K: CateIndex<Data=V> + IdxContains<T>, V: IndexedBy<K> + Contains<T> + 'static, T:'static;

impl<'a,K,V,T> Iterator for IterCate<'a, K, V, T>
where K: CateIndex<Data=V> + IdxContains<T>, V: IndexedBy<K> + Contains<T> + 'static, T:'static
{
  type Item = (K, &'a T);
  fn next(&mut self) -> Option<Self::Item> {
    self.0.next().and_then(|(id, data)| Some((<K as IdxContains::<T>>::wrap(*id), data)))
  }
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.0.size_hint()
  }
}

impl<'a,K,V,T> FusedIterator for IterCate<'a, K, V, T>
where K: CateIndex<Data=V> + IdxContains<T>, V: IndexedBy<K> + Contains<T> + 'static, T:'static {}

impl<'a,K,V,T> ExactSizeIterator for IterCate<'a, K, V, T>
where K: CateIndex<Data=V> + IdxContains<T>, V: IndexedBy<K> + Contains<T> + 'static, T:'static {}


#[derive(Debug, Default)]
pub struct IterCateMut<'a, K, V, T>(map::IterMut<'a, usize, T>, PhantomData<K>, PhantomData<V>) where 
  K: CateIndex<Data=V> + IdxContains<T>, V: IndexedBy<K> + Contains<T> + 'static, T:'static;


impl<'a,K,V,T> Iterator for IterCateMut<'a, K, V, T>
where K: CateIndex<Data=V> + IdxContains<T>, V: IndexedBy<K> + Contains<T> + 'static, T:'static {
  type Item = (K, &'a mut T);
  fn next(&mut self) -> Option<Self::Item> {
    self.0.next().and_then(|(id, data)| Some((<K as IdxContains::<T>>::wrap(*id), data)))
  }
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.0.size_hint()
  }
}

impl<'a,K,V,T> FusedIterator for IterCateMut<'a, K, V, T>
where K: CateIndex<Data=V> + IdxContains<T>, V: IndexedBy<K> + Contains<T> + 'static, T:'static {}

impl<'a,K,V,T> ExactSizeIterator for IterCateMut<'a, K, V, T>
where K: CateIndex<Data=V> + IdxContains<T>, V: IndexedBy<K> + Contains<T> + 'static, T:'static {}

