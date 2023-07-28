use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CtorId(pub u64);
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstId(pub u64);
#[derive(Serialize, Deserialize, Debug)]
pub struct CtorCall {
  pub ctor: CtorId,
}
pub type InstRef = Vec<InstId>;
#[derive(Serialize, Deserialize, Debug)]
pub struct Connection {
  pub left: InstRef,
  pub right: InstRef,
}
type Sym = String;
#[derive(Serialize, Deserialize, Debug)]
pub struct ReactorCtor {
  pub inst2sym: HashMap<InstId, Sym>,
  pub insts: HashMap<InstId, CtorCall>,
  pub connections: Vec<Connection>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct BinaryCtor {
  pub path: PathBuf,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum Ctor {
  ReactorCtor(ReactorCtor),
  BinaryCtor(BinaryCtor),
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Program {
  pub ctor2sym: HashMap<CtorId, Sym>,
  pub ctors: HashMap<CtorId, Ctor>,
  pub main: CtorId,
}
