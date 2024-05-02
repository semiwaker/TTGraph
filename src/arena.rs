use std::collections::{btree_map, BTreeMap};
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::{ExactSizeIterator, FusedIterator, Iterator};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub trait ArenaIndex:
  Hash + PartialEq + Eq + Debug + Copy + Clone + PartialOrd + Ord
{
  fn new(id: usize) -> Self;
}

/// The internal conatiner for nodes.
/// Currently, it is a wrapper over BTreeMap.
#[derive(Debug, Clone)]
pub(crate) struct Arena<K: ArenaIndex, V> {
  distributer: Arc<IdDistributer>,
  container: BTreeMap<K, V>,
}

impl<K: ArenaIndex, V> Arena<K, V> {
  pub fn new(distributer: Arc<IdDistributer>) -> Self {
    Arena { distributer, container: BTreeMap::new() }
  }

  // pub fn clear(&mut self) {
  //   self.container.clear()
  // }

  pub fn insert(&mut self, item: V) -> K {
    let idx = K::new(self.alloc_id());
    self.container.insert(idx, item);
    idx
  }

  // pub fn insert_with(&mut self, create: impl FnOnce(K) -> T) -> K {
  //   let idx = K::new(self.alloc_id());
  //   self.container.insert(idx, create(idx));
  //   idx
  // }

  pub fn alloc(&mut self) -> K {
    K::new(self.alloc_id())
  }

  pub fn fill_back(&mut self, i: K, item: V) {
    assert!(self.container.insert(i, item).is_none(), "Filled an illegal index!")
  }

  pub fn remove(&mut self, i: K) -> Option<V> {
    self.container.remove(&i)
  }

  pub fn contains(&self, i: K) -> bool {
    self.container.contains_key(&i)
  }

  pub fn get(&self, i: K) -> Option<&V> {
    self.container.get(&i)
  }

  pub fn get_mut(&mut self, i: K) -> Option<&mut V> {
    self.container.get_mut(&i)
  }

  pub fn update_with<F>(&mut self, i: K, f: F)
  where
    F: FnOnce(V) -> V,
  {
    let x = self.container.remove(&i).unwrap();
    self.container.insert(i, f(x));
  }

  pub fn len(&self) -> usize {
    self.container.len()
  }

  // pub fn is_empty(&self) -> bool {
  //   self.container.is_empty()
  // }

  pub fn merge(&mut self, other: Arena<K, V>) {
    for (idx, value) in other {
      self.fill_back(idx, value);
    }
  }

  pub fn iter(&self) -> Iter<'_, K, V> {
    Iter(self.container.iter())
  }

  pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
    IterMut(self.container.iter_mut())
  }

  fn alloc_id(&self) -> usize {
    self.distributer.alloc()
  }
}

#[derive(Debug)]
pub struct Iter<'a, K, V>(btree_map::Iter<'a, K, V>)
where
  K: ArenaIndex,
  V: 'a;

impl<'a, K, V> Clone for Iter<'a, K, V>
where
  K: ArenaIndex,
  V: 'a,
{
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
  K: ArenaIndex,
  V: 'a,
{
  type Item = (K, &'a V);
  fn next(&mut self) -> Option<Self::Item> {
    self.0.next().and_then(|(idx, data)| Some((*idx, data)))
  }
}
impl<'a, K, V> FusedIterator for Iter<'a, K, V>
where
  K: ArenaIndex,
  V: 'a,
{
}
impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V>
where
  K: ArenaIndex,
  V: 'a,
{
  fn len(&self) -> usize {
    self.0.len()
  }
}
#[derive(Debug)]
pub struct IntoIter<K, V>(btree_map::IntoIter<K, V>)
where
  K: ArenaIndex;

impl<K, V> Iterator for IntoIter<K, V>
where
  K: ArenaIndex,
{
  type Item = (K, V);
  fn next(&mut self) -> Option<Self::Item> {
    self.0.next()
  }
}

impl<K, V> FusedIterator for IntoIter<K, V> where K: ArenaIndex {}

#[derive(Debug)]
pub struct IterMut<'a, K, V>(btree_map::IterMut<'a, K, V>)
where
  V: 'a,
  K: ArenaIndex;

impl<'a, K, V> Iterator for IterMut<'a, K, V>
where
  V: 'a,
  K: ArenaIndex,
{
  type Item = (K, &'a mut V);
  fn next(&mut self) -> Option<Self::Item> {
    self.0.next().and_then(|(idx, data)| Some((*idx, data)))
  }
}

impl<'a, K, V> FusedIterator for IterMut<'a, K, V>
where
  V: 'a,
  K: ArenaIndex,
{
}

impl<K, V> IntoIterator for Arena<K, V>
where
  K: ArenaIndex,
{
  type IntoIter = IntoIter<K, V>;
  type Item = (K, V);

  fn into_iter(self) -> Self::IntoIter {
    IntoIter(self.container.into_iter())
  }
}
impl<'a, K, V> IntoIterator for &'a Arena<K, V>
where
  V: 'a,
  K: ArenaIndex,
{
  type IntoIter = Iter<'a, K, V>;
  type Item = (K, &'a V);

  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

impl<'a, K, V> IntoIterator for &'a mut Arena<K, V>
where
  V: 'a,
  K: ArenaIndex,
{
  type IntoIter = IterMut<'a, K, V>;
  type Item = (K, &'a mut V);

  fn into_iter(self) -> Self::IntoIter {
    self.iter_mut()
  }
}

impl<K: ArenaIndex, V> std::ops::Index<K> for Arena<K, V> {
  type Output = V;

  fn index(&self, index: K) -> &Self::Output {
    self.get(index).unwrap()
  }
}
impl<K: ArenaIndex, V> std::ops::IndexMut<K> for Arena<K, V> {
  fn index_mut(&mut self, index: K) -> &mut Self::Output {
    self.get_mut(index).unwrap()
  }
}

/// An atomic context to make distinct usize ids
#[derive(Debug)]
pub struct IdDistributer {
  cnt: AtomicUsize,
}

impl IdDistributer {
  pub fn new() -> IdDistributer {
    IdDistributer { cnt: AtomicUsize::new(0) }
  }

  pub fn alloc(&self) -> usize {
    let c = self.cnt.fetch_add(1, Ordering::Relaxed);
    c + 1
  }

  pub(crate) fn from_count(cnt: usize) -> IdDistributer {
    IdDistributer { cnt: AtomicUsize::new(cnt) }
  }
}

impl Default for IdDistributer {
  fn default() -> Self {
    IdDistributer { cnt: AtomicUsize::new(0) }
  }
}
