use crate::ir_serializable::{
  BinaryCtor, Connection, Ctor, CtorCall, CtorId, DebugOnlyId, InstId, InstRef, Program,
  StructlikeCtor,
};
use std::{collections::HashMap, fmt::Display};

impl Display for CtorId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "0x{:x}", self.0)
  }
}

impl Display for InstId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Display for DebugOnlyId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Display for CtorCall {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.ctor)
  }
}

impl Display for InstRef {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      self.0.iter().fold(String::new(), |s, e| if s.is_empty() {
        e.to_string()
      } else {
        s + "." + &e.to_string()
      })
    )
  }
}

impl Display for Connection {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} {} {}", self.id, self.left, self.right)
  }
}

fn sortedkeys<K: Ord, V>(hm: &HashMap<K, V>) -> Vec<&K> {
  let mut sorted: Vec<&K> = hm.keys().collect();
  sorted.sort();
  sorted
}

fn print_tokenlist<T: Display>(v: &Vec<T>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
  if !v.is_empty() {
    write!(f, " ")?;
    for id in v {
      write!(f, " {id}")?;
    }
    writeln!(f)?;
  }
  Ok(())
}

impl Display for StructlikeCtor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for iid in sortedkeys(&self.inst2sym) {
      writeln!(f, "  {} {} = {}", self.inst2sym[iid], iid, self.insts[iid])?;
    }
    writeln!(f, "  ---")?;
    print_tokenlist(&self.left, f)?;
    writeln!(f, "  ---")?;
    print_tokenlist(&self.right, f)?;
    writeln!(f, "  ---")?;
    for connection in &self.connections {
      writeln!(f, "  {connection}")?;
    }
    Ok(())
  }
}

impl Display for BinaryCtor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.path.display())
  }
}

impl Display for Program {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for (cid, bc) in sortedkeys(&self.ctorid2sym)
      .iter()
      .filter_map(|cid| match &self.ctors[cid] {
        Ctor::BinaryCtor(bc) => Some((cid, bc)),
        Ctor::StructlikeCtor(_) => None,
      })
    {
      let sym: &str = &self.ctorid2sym[cid];
      writeln!(f, "{sym} {cid} {bc}")?;
    }
    writeln!(f, "---")?;
    for (cid, rc) in sortedkeys(&self.ctorid2sym)
      .iter()
      .filter_map(|cid| match &self.ctors[cid] {
        Ctor::StructlikeCtor(rc) => Some((cid, rc)),
        Ctor::BinaryCtor(_) => None,
      })
    {
      let sym: &str = &self.ctorid2sym[cid];
      write!(f, "{sym} {cid}\n{rc}")?;
    }
    writeln!(f, "---")?;
    writeln!(f, "{}", self.main)
  }
}
