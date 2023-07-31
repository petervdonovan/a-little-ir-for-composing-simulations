use irlf_ser::ir::{CtorId, DebugOnlyId, InstId};
use typed_arena::Arena;
// use std::{collections::HashMap, path::PathBuf};

// use serde::{Deserialize, Serialize};

// pub struct CtorId(pub u64);

use std::{cell::OnceCell, collections::HashMap, path::PathBuf};

// pub struct CtorCall {
//   pub ctor: CtorId,
// }
#[derive(Debug)]
pub struct CtorCall<'a> {
  #[cfg(debug_assertions)]
  pub id: InstId,
  pub ctor: &'a Ctor<'a>,
}

// pub struct InstRef(pub Vec<InstId>);
#[derive(Debug)]
pub struct InstRef<'a>(pub Vec<&'a CtorCall<'a>>);

// pub struct Connection {
//   pub left: InstRef,
//   pub right: InstRef,
// }
#[derive(Debug)]
pub struct Connection<'a> {
  #[cfg(debug_assertions)]
  pub id: DebugOnlyId,
  pub left: InstRef<'a>,
  pub right: InstRef<'a>,
}

// pub struct StructlikeCtor {
//   pub inst2sym: HashMap<InstId, Sym>,
//   pub insts: HashMap<InstId, CtorCall>,
//   pub connections: Vec<Connection>,
// }
#[derive(Debug)]
pub struct StructlikeCtor<'a> {
  pub body: OnceCell<StructlikeCtorBody<'a>>,
}

#[derive(Debug)]
pub struct StructlikeCtorBody<'a> {
  pub insts: Vec<&'a CtorCall<'a>>,
  pub body: OnceCell<StructlikeCtorBodyBody<'a>>,
}

#[derive(Debug)]
pub struct StructlikeCtorBodyBody<'a> {
  pub connections: Vec<Connection<'a>>,
}

// pub struct BinaryCtor {
//   pub path: PathBuf,
// }
#[derive(Debug)]
pub struct BinaryCtor {
  pub path: PathBuf,
}

// pub enum Ctor {
//   ReactorCtor(ReactorCtor),
//   BinaryCtor(BinaryCtor),
// }
#[derive(Debug)]
pub enum CtorImpl<'a> {
  StructlikeCtor(&'a StructlikeCtor<'a>),
  BinaryCtor(&'a BinaryCtor),
}
#[derive(Debug)]
pub struct Ctor<'a> {
  #[cfg(debug_assertions)]
  pub id: CtorId,
  pub imp: CtorImpl<'a>,
}

// pub struct Program {
//   pub ctor2sym: HashMap<CtorId, Sym>,
//   pub ctors: HashMap<CtorId, Ctor>,
//   pub main: CtorId,
// }
#[derive(Debug)]
pub struct Program<'a> {
  // insts_arena: Arena<CtorCall<'a>>,
  // ctors_arena: Arena<Ctor<'a>>,
  pub ctors: Vec<&'a Ctor<'a>>,
  pub main: &'a Ctor<'a>,
}

pub struct Arenas<'a> {
  pub insts_arena: Arena<CtorCall<'a>>,
  pub binary_ctors_arena: Arena<BinaryCtor>,
  pub structlike_ctors_arena: Arena<StructlikeCtor<'a>>,
  pub ctors_arena: Arena<Ctor<'a>>,
}
