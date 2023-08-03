use irlf_ser::ir::{CtorId, DebugOnlyId, InstId};
use typed_arena::Arena;

use std::{cell::OnceCell, path::PathBuf};

pub type Iface<'a> = Vec<&'a CtorCall<'a>>;

#[derive(Debug)]
pub struct CtorCall<'a> {
  #[cfg(debug_assertions)]
  pub id: InstId,
  pub ctor: &'a Ctor<'a>,
}

#[derive(Debug)]
pub struct InstRef<'a>(pub Vec<&'a CtorCall<'a>>);

#[derive(Debug)]
pub struct Connection<'a> {
  #[cfg(debug_assertions)]
  pub id: DebugOnlyId,
  pub left: InstRef<'a>,
  pub right: InstRef<'a>,
}

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
  pub left: Iface<'a>,
  pub right: Iface<'a>,
  pub connections: Vec<Connection<'a>>,
}

#[derive(Debug)]
pub struct BinaryCtor {
  pub path: PathBuf,
}

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

#[derive(Debug)]
pub struct Program<'a> {
  pub ctors: Vec<&'a Ctor<'a>>,
  pub main: &'a Ctor<'a>,
}

pub struct Arenas<'a> {
  pub insts_arena: Arena<CtorCall<'a>>,
  pub binary_ctors_arena: Arena<BinaryCtor>,
  pub structlike_ctors_arena: Arena<StructlikeCtor<'a>>,
  pub ctors_arena: Arena<Ctor<'a>>,
}

impl<'a> Arenas<'a> {
  #[must_use]
  pub fn new() -> Arenas<'a> {
    Arenas {
      insts_arena: Arena::new(),
      binary_ctors_arena: Arena::new(),
      structlike_ctors_arena: Arena::new(),
      ctors_arena: Arena::new(),
    }
  }
}

impl<'a> Default for Arenas<'a> {
  fn default() -> Self {
    Arenas::new()
  }
}
