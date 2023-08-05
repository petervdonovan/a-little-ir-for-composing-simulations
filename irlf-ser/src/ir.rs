use std::{collections::HashMap, path::PathBuf};

use lf_types::{CtorId, DebugOnlyId, Iface, InstId};
use serde::{Deserialize, Serialize};
pub type IfaceElt = InstId;
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
pub type Sym = String;
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct StructlikeCtor {
  pub inst2sym: HashMap<InstId, Sym>,
  pub insts: HashMap<InstId, CtorCall>,
  pub iface: Iface<IfaceElt>,
  pub connections: Vec<Connection>,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct BinaryCtor {
  pub path: PathBuf,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct LibCtor {
  pub name: String,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Ctor {
  StructlikeCtor(StructlikeCtor),
  BinaryCtor(BinaryCtor),
  LibCtor(LibCtor),
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Program {
  pub ctorid2sym: HashMap<CtorId, Sym>,
  pub ctors: HashMap<CtorId, Ctor>,
  pub main: CtorId,
}
