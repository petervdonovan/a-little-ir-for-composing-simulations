use crate::ir_serializable::*;
use std::{collections::HashMap, fmt::Display};

impl Display for CtorId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Display for InstId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Display for CtorCall {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.ctor)
  }
}

fn display_instref(iref: &InstRef) -> String {
  iref
    .iter()
    .fold(String::new(), |s, e| s + "." + &e.to_string())
}

impl Display for Connection {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{} {}",
      display_instref(&self.left),
      display_instref(&self.right)
    )
  }
}

fn sortedkeys<K: Ord, V>(hm: &HashMap<K, V>) -> Vec<&K> {
  let mut sorted: Vec<&K> = hm.keys().collect();
  sorted.sort();
  sorted
}

impl Display for ReactorCtor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for iid in sortedkeys(&self.inst2sym) {
      writeln!(f)?;
      write!(f, "  {} {} = {}", iid, self.inst2sym[iid], self.insts[iid])?;
    }
    for connection in self.connections.iter() {
      writeln!(f)?;
      write!(f, "{}", connection)?;
    }
    Ok(())
  }
}

impl Display for BinaryCtor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "import {}", self.path.display())
  }
}

impl Display for Program {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for bc in sortedkeys(&self.ctor2sym)
      .iter()
      .filter_map(|cid| match &self.ctors[cid] {
        Ctor::BinaryCtor(bc) => Some(bc),
        _ => None,
      })
    {
      writeln!(f, "{}", bc)?;
    }

    for rc in sortedkeys(&self.ctor2sym)
      .iter()
      .filter_map(|cid| match &self.ctors[cid] {
        Ctor::ReactorCtor(rc) => Some(rc),
        _ => None,
      })
    {
      writeln!(f, "{}", rc)?;
    }
    Ok(())
  }
}
