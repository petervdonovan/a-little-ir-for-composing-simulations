use std::fmt::Display;

use derive_more::{Add, AddAssign};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Side {
  Left,
  Right,
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct IfaceNode<IfaceElt: std::hash::Hash>(pub Side, pub IfaceElt);
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

impl<IfaceElt: Display + std::hash::Hash> Display for IfaceNode<IfaceElt> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} {}", self.0, self.1)
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
