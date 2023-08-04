pub mod convert;
pub mod ir;
pub mod unconvert;

#[salsa::jar(db = Db)]
pub struct Jar(
  crate::ir::SourceProgram,
  crate::ir::Program,
  crate::ir::BinaryCtor,
  crate::ir::StructlikeCtor,
  crate::ir::LibCtor,
  crate::ir::Inst,
  crate::ir::Connection,
  crate::convert::convert,
  crate::ir::InstRef,
  crate::ir::Id2Sym,
);

pub trait Db: salsa::DbWithJar<Jar> {}

impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> {}
