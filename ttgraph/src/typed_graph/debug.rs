use std::fmt;
use std::fmt::Debug;

use super::*;

impl<T: NodeEnum + Debug> Debug for Graph<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "Graph {{")?;
    writeln!(f, "  ctx_id = {:?},", self.ctx_id)?;
    writeln!(f, "  nodes = [")?;
    for (i, n) in self.iter() {
      writeln!(f, "    {:?}: {:?}", i, n)?;
    }
    writeln!(f, "  ],")?;
    writeln!(f, "  back_link=[")?;
    for (i, s) in self.back_links.iter() {
      writeln!(f, "    {:?}: {:?}", i, s)?;
    }
    writeln!(f, "  ]")?;
    writeln!(f, "}}")?;
    Ok(())
  }
}
