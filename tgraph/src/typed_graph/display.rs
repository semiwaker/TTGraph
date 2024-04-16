use std::fmt;
use std::fmt::Display;

use super::*;

impl<T: NodeEnum + Display> Display for Graph<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "Graph {{")?;
    for (i, n) in self.iter() {
      writeln!(f, "  {}: {}", i, n)?;
    }
    writeln!(f, "}}")?;
    Ok(())
  }
}
