#![cfg(feature = "debug")]
use super::*;
use std::collections::BTreeMap;

pub type CheckResult = Result<(), ()>;
pub type NodeCheckFunc<NodeT> = Box<dyn (Fn(NodeIndex, &NodeT) -> CheckResult) + 'static>;
pub type LinkCheckFunc<NodeT> =
  Box<dyn (Fn(NodeIndex, NodeIndex, &NodeT, Option<&NodeT>) -> CheckResult) + 'static>;

/// A container for check functions
/// + Node check: `|idx, &node| -> CheckResult`, applies when a node have been changed or newly inserted
/// + Link add check: `|idx_from, idx_to, &node_from, Option<&node_to>| -> CheckResult`, applies when a link have been added into the graph.
/// + Link remove check: `|idx_from, idx_to, &node_from, Option<&node_to>| -> CheckResult`, applies when a link have been removed from the graph.
pub struct GraphCheck<NodeT: NodeEnum> {
  pub(crate) node_checks: BTreeMap<String, NodeCheckFunc<NodeT>>,
  pub(crate) link_add_checks: BTreeMap<String, LinkCheckFunc<NodeT>>,
  pub(crate) link_remove_checks: BTreeMap<String, LinkCheckFunc<NodeT>>,
}

impl<NodeT: NodeEnum> GraphCheck<NodeT> {
  pub fn new() -> Self {
    GraphCheck {
      node_checks: BTreeMap::new(),
      link_add_checks: BTreeMap::new(),
      link_remove_checks: BTreeMap::new(),
    }
  }

  pub fn insert_node_check(
    &mut self, name: String, func: impl Fn(NodeIndex, &NodeT) -> CheckResult + 'static,
  ) {
    self.node_checks.insert(name, Box::new(func));
  }

  pub fn remove_node_check(&mut self, name: &str) {
    self.node_checks.remove(name);
  }

  pub fn insert_link_add_check(
    &mut self, name: String,
    func: impl Fn(NodeIndex, NodeIndex, &NodeT, Option<&NodeT>) -> CheckResult + 'static,
  ) {
    self.link_add_checks.insert(name, Box::new(func));
  }

  pub fn remove_link_add_check(&mut self, name: &str) {
    self.link_add_checks.remove(name);
  }

  pub fn insert_link_remove_check(
    &mut self, name: String,
    func: impl Fn(NodeIndex, NodeIndex, &NodeT, Option<&NodeT>) -> CheckResult + 'static,
  ) {
    self.link_remove_checks.insert(name, Box::new(func));
  }

  pub fn remove_link_remove_check(&mut self, name: &str) {
    self.link_remove_checks.remove(name);
  }
}

impl<NodeT: NodeEnum> Default for GraphCheck<NodeT> {
  fn default() -> Self {
    Self::new()
  }
}
