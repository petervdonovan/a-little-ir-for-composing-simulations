use std::{collections::HashMap, path::PathBuf};

pub type IfaceElt = InstRef;

use lf_types::{CtorId, DebugOnlyId, Iface, InstId};

#[salsa::tracked]
pub struct Inst {
  #[id]
  pub id: InstId,
  #[return_ref]
  pub ctor: Ctor,
}

#[salsa::tracked]
pub struct InstRef {
  #[id]
  pub iref: Vec<Inst>,
}

#[salsa::tracked]
pub struct Connection {
  #[id]
  pub id: DebugOnlyId,
  #[return_ref]
  pub left: InstRef,
  #[return_ref]
  pub right: InstRef,
}

#[salsa::tracked]
pub struct StructlikeCtor {
  #[id]
  pub id: CtorId,
  #[return_ref]
  pub insts: Vec<Inst>,
  #[return_ref]
  pub iface: Iface<IfaceElt>,
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
