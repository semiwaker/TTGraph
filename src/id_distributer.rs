use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

/// An atomic context to make distinct usize ids
#[derive(Debug, Clone)]
pub struct IdDistributer {
  cnt: Arc<AtomicUsize>,
}

impl IdDistributer {
  pub fn new() -> IdDistributer {
    IdDistributer { cnt: Arc::new(AtomicUsize::new(0)) }
  }

  pub fn alloc(&self) -> usize {
    let c = self.cnt.fetch_add(1, Ordering::Relaxed);
    c + 1
  }

  pub(crate) fn from_count(cnt: usize) -> IdDistributer {
    IdDistributer { cnt: Arc::new(AtomicUsize::new(cnt)) }
  }
}

impl Default for IdDistributer {
  fn default() -> Self {
    IdDistributer { cnt: Arc::new(AtomicUsize::new(0)) }
  }
}

