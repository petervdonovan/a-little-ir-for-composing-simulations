use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct CtorId(pub u64);
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct InstId(pub u64);
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct DebugOnlyId(pub u64);
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CtorCall {
  pub ctor: CtorId,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct InstRef(pub Vec<InstId>);
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Connection {
  pub id: DebugOnlyId,
  pub left: InstRef,
  pub right: InstRef,
}
type Sym = String;
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct StructlikeCtor {
  pub inst2sym: HashMap<InstId, Sym>,
  pub insts: HashMap<InstId, CtorCall>,
  pub left: Vec<InstId>,
  pub right: Vec<InstId>,
  pub connections: Vec<Connection>,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct BinaryCtor {
  pub path: PathBuf,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Ctor {
  StructlikeCtor(StructlikeCtor),
  BinaryCtor(BinaryCtor),
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Program {
  pub ctorid2sym: HashMap<CtorId, Sym>,
  pub ctors: HashMap<CtorId, Ctor>,
  pub main: CtorId,
}
