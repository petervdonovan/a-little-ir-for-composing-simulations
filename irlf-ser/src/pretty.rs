use crate::ir::{
  BinaryCtor, Connection, Ctor, CtorCall, InstRef, LibCtor, Program, StructlikeCtor,
};
use std::{collections::HashMap, fmt::Display};

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
    print_tokenlist(&self.iface, f)?;
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

impl Display for LibCtor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.name)
  }
}

macro_rules! visit_ctor {
  ($self: ident, $CtorVariant: ident, $id: ident, $ctor: ident, $b: block) => {
    for ($id, $ctor) in
      sortedkeys(&$self.ctorid2sym)
        .iter()
        .filter_map(|cid| match &$self.ctors[cid] {
          Ctor::$CtorVariant(bc) => Some((cid, bc)),
          _ => None,
        })
    {
      $b
    }
  };
}

impl Display for Program {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    visit_ctor!(self, LibCtor, cid, lctor, {
      let sym: &str = &self.ctorid2sym[cid];
      writeln!(f, "{sym} {cid} {lctor}")?;
    });
    writeln!(f, "---")?;
    visit_ctor!(self, BinaryCtor, cid, bc, {
      let sym: &str = &self.ctorid2sym[cid];
      writeln!(f, "{sym} {cid} {bc}")?;
    });
    writeln!(f, "---")?;
    visit_ctor!(self, StructlikeCtor, cid, sctor, {
      let sym: &str = &self.ctorid2sym[cid];
      write!(f, "{sym} {cid}\n{sctor}")?;
    });
    writeln!(f, "---")?;
    writeln!(f, "{}", self.main)
  }
}
