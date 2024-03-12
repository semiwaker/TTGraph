use std::collections::btree_set;
use std::iter::Iterator;

use super::*;

pub struct EdgeIterator<'a: 'b, 'b, EDataT> {
  pub edges: &'a Arena<Edge<EDataT>, EdgeIndex>,
  pub iter: btree_set::Iter<'b, EdgeIndex>,
}
impl<'a: 'b, 'b, EDataT> Iterator for EdgeIterator<'a, 'b, EDataT> {
  type Item = (EdgeIndex, NodeIndex);

  fn next(&mut self) -> Option<Self::Item> {
    let e = self.iter.next();
    e.map(|idx| (*idx, self.edges.get(*idx).unwrap().from))
  }
}
