use std::collections::{
    hash_map::{self, Iter, IterMut},
    HashMap,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Index {
    pub id: usize,
}
impl Index {
    pub fn new(id: usize) -> Index {
        Index { id }
    }
    pub fn placeholder() -> Index {
        Index { id: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct Arena<T> {
    id_cnt: usize,
    container: HashMap<usize, T>,
}

impl<T> Arena<T> {
    pub fn new() -> Arena<T> {
        Arena {
            id_cnt: 0,
            container: HashMap::new(),
        }
    }
    pub fn clear(&mut self) {
        self.container.clear()
    }

    pub fn insert(&mut self, item: T) -> Index {
        let id = self.alloc_id();
        self.container.insert(id, item);
        Index { id }
    }
    pub fn insert_with(&mut self, create: impl FnOnce(Index) -> T) -> Index {
        let id = self.alloc_id();
        self.container.insert(id, create(Index { id }));
        Index { id }
    }

    pub fn fill_back(&mut self, i: Index, item: T) {
        assert!(
            self.container.insert(i.id, item).is_none(),
            "tgraph::arena::fill_back: filled an illegal index!"
        )
    }

    pub fn remove(&mut self, i: Index) -> Option<T> {
        self.container.remove(&i.id)
    }

    pub fn contains(&self, i: Index) -> bool {
        self.container.contains_key(&i.id)
    }

    pub fn get(&self, i: Index) -> Option<&T> {
        self.container.get(&i.id)
    }
    pub fn get_mut(&mut self, i: Index) -> Option<&mut T> {
        self.container.get_mut(&i.id)
    }

    pub fn update_with<F>(&mut self, i: Index, f: F)
    where
        F: FnOnce(T) -> T,
    {
        let x = self.container.remove(&i.id).unwrap();
        self.container.insert(i.id, f(x));
    }

    pub fn len(&self) -> usize {
        self.container.len()
    }
    pub fn is_empty(&self) -> bool {
        self.container.is_empty()
    }

    pub fn alloc_for_merge<U>(&mut self, other: &Arena<U>) -> HashMap<Index, Index> {
        let mut map = HashMap::new();
        for (k, _) in other.iter() {
            let id = self.alloc_id();
            map.insert(Index { id: *k }, Index { id });
        }
        map
    }

    pub fn iter(&self) -> Iter<'_, usize, T> {
        self.container.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, usize, T> {
        self.container.iter_mut()
    }

    fn alloc_id(&mut self) -> usize {
        self.id_cnt += 1;
        self.id_cnt
    }
}

impl<T> IntoIterator for Arena<T> {
    type Item = (usize, T);
    type IntoIter = hash_map::IntoIter<usize, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.container.into_iter()
    }
}
impl<'a, T> IntoIterator for &'a Arena<T> {
    type Item = (&'a usize, &'a T);
    type IntoIter = Iter<'a, usize, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.container.iter()
    }
}
impl<'a, T> IntoIterator for &'a mut Arena<T> {
    type Item = (&'a usize, &'a mut T);
    type IntoIter = IterMut<'a, usize, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.container.iter_mut()
    }
}

impl<T> std::ops::Index<Index> for Arena<T> {
    type Output = T;
    fn index(&self, index: Index) -> &Self::Output {
        self.get(index).unwrap()
    }
}
impl<T> std::ops::IndexMut<Index> for Arena<T> {
    fn index_mut(&mut self, index: Index) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Arena::new()
    }
}
