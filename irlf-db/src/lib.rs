mod convert;
mod ir;
mod unconvert;

#[salsa::jar(db = Db)]
pub struct Jar(
  crate::ir::SourceProgram,
  crate::ir::Program,
  crate::ir::BinaryCtor,
  crate::ir::StructlikeCtor,
  crate::ir::Inst,
  crate::ir::Connection,
  crate::convert::convert,
  crate::ir::InstRef,
  crate::ir::Id2Sym,
);

pub trait Db: salsa::DbWithJar<Jar> {}

impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> {}
