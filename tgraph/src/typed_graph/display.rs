use std::fmt;
use std::fmt::Display;

use super::*;

impl<T: NodeEnum + Debug> Display for Graph<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "Graph {{")?;
    for (i, n) in self.iter_nodes() {
      writeln!(f, "  {}: {:?}", i.0, n)?;
    }
    writeln!(f, "}}")?;
    Ok(())
  }
}
