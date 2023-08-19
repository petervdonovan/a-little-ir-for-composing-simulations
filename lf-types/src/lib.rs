use std::fmt::Display;
use std::hash::Hash;

use derive_more::{Add, AddAssign};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Side {
  Left,
  Right,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SideMatch {
  One(Side),
  Both,
}

impl SideMatch {
  pub fn includes(&self, side: Side) -> bool {
    match self {
      SideMatch::One(s) => *s == side,
      SideMatch::Both => true,
    }
  }
  pub fn overlaps(&self, other: Self) -> bool {
    match self {
      SideMatch::One(s) => other.includes(*s),
      SideMatch::Both => true,
    }
  }
}

pub enum Nesting {
  Up,
  Down,
}
#[derive(
  Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord, Add, AddAssign,
)]
pub struct Level(pub u32);

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum FlowDirection {
  In,
  Out,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub enum Comm<IfaceElt> {
  Notify,
  Data(IfaceElt),
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct IfaceNode<IfaceElt>(pub SideMatch, pub Comm<IfaceElt>);
pub type Iface<IfaceElt> = Vec<IfaceNode<IfaceElt>>;

pub type DeltaT = Vec<u64>;
pub type Net = DeltaT;

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct CtorId(pub u64);
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct InstId(pub u64);
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct DebugOnlyId(pub u64);

impl Display for Side {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Left => write!(f, "L"),
      Self::Right => write!(f, "R"),
    }
  }
}

impl Display for SideMatch {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      SideMatch::One(s) => s.fmt(f),
      SideMatch::Both => write!(f, "A"),
    }
  }
}

impl<IfaceElt: Display> Display for IfaceNode<IfaceElt> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} {}", self.0, self.1)
  }
}

impl<IfaceEltA> Comm<IfaceEltA> {
  pub fn map<IfaceEltB>(&self, f: impl Fn(&IfaceEltA) -> IfaceEltB) -> Comm<IfaceEltB> {
    match self {
      Comm::Notify => Comm::Notify,
      Comm::Data(x) => Comm::Data(f(x)),
    }
  }
  pub fn unwrap(self) -> IfaceEltA {
    match self {
      Comm::Notify => panic!(),
      Comm::Data(x) => x,
    }
  }
}

impl<IfaceElt: Display> Display for Comm<IfaceElt> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Comm::Notify => write!(f, "-"),
      Comm::Data(k) => k.fmt(f),
    }
  }
}

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
