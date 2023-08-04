use std::{collections::HashMap, path::PathBuf};

pub type Iface = Vec<Inst>;

use irlf_ser::ir::{CtorId, DebugOnlyId, InstId};

#[salsa::interned]
pub struct Inst {
  pub id: InstId,
  #[return_ref]
  pub ctor: Ctor,
}

#[salsa::interned]
pub struct InstRef {
  pub iref: Vec<Inst>,
}

#[salsa::interned]
pub struct Connection {
  pub id: DebugOnlyId,
  #[return_ref]
  pub left: InstRef,
  #[return_ref]
  pub right: InstRef,
}

#[salsa::interned]
pub struct StructlikeCtor {
  pub id: CtorId,
  // pub inst2sym: HashMap<InstId, Sym>,
  #[return_ref]
  pub insts: Vec<Inst>,
  #[return_ref]
  pub left: Iface,
  #[return_ref]
  pub right: Iface,
  #[return_ref]
  pub connections: Vec<Connection>,
}

#[salsa::tracked]
pub struct BinaryCtor {
  #[id]
  pub id: CtorId,
  pub path: PathBuf, // TODO: make this an input. For example, use salsa::input and include the "date last modified" as part of this struct, kind of like how Make does it.
}

#[salsa::tracked]
pub struct LibCtor {
  #[id]
  pub id: CtorId,
  #[return_ref]
  pub name: String,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Ctor {
  StructlikeCtor(StructlikeCtor),
  BinaryCtor(BinaryCtor),
  LibCtor(LibCtor),
}

#[salsa::tracked]
pub struct Program {
  // pub ctorid2sym: HashMap<CtorId, Sym>,
  #[return_ref]
  pub ctors: Vec<Ctor>,
  #[return_ref]
  pub main: Ctor,
}

#[salsa::input]
pub struct SourceProgram {
  #[return_ref]
  pub source: irlf_ser::ir::Program,
}

#[salsa::tracked]
pub struct Id2Sym {
  #[return_ref]
  pub inst2sym: HashMap<InstId, irlf_ser::ir::Sym>,
  #[return_ref]
  pub ctor2sym: HashMap<CtorId, irlf_ser::ir::Sym>,
}
