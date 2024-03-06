use std::fmt;
use std::fmt::Display;

use super::*;

impl<T: NodeEnum + Debug> Display for Graph<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Graph {{\n")?;
    for (i, n) in self.iter_nodes() {
      write!(f, "  {}: {:?}\n", i.0, n)?;
    }
    write!(f, "}}\n")?;
    Ok(())
  }
}
