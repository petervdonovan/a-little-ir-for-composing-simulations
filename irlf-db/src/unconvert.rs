use std::collections::HashMap;

use irlf_ser::ir::CtorId;

use crate::Db;

impl crate::ir::Ctor {
  fn id(&self, db: &dyn Db) -> CtorId {
    match self {
      crate::ir::Ctor::StructlikeCtor(sctor) => sctor.id(db),
      crate::ir::Ctor::BinaryCtor(bctor) => bctor.id(db),
      crate::ir::Ctor::LibCtor(lctor) => lctor.id(db),
    }
  }
}

pub fn unconvert(
  db: &dyn Db,
  program: crate::ir::Program,
  id2sym: crate::ir::Id2Sym,
) -> irlf_ser::ir::Program {
  irlf_ser::ir::Program {
    ctorid2sym: id2sym.ctor2sym(db).clone(),
    ctors: unconvert_ctors(db, &program, &id2sym),
    main: program.main(db).id(db),
  }
}

fn unconvert_ctors(
  db: &dyn Db,
  program: &crate::ir::Program,
  id2sym: &crate::ir::Id2Sym,
) -> HashMap<irlf_ser::ir::CtorId, irlf_ser::ir::Ctor> {
  let mut acc = HashMap::new();
  for ctor in program.ctors(db) {
    acc.insert(ctor.id(db), unconvert_ctor(db, ctor, id2sym));
  }
  acc
}

fn unconvert_ctor(
  db: &dyn Db,
  ctor: &crate::ir::Ctor,
  id2sym: &crate::ir::Id2Sym,
) -> irlf_ser::ir::Ctor {
  match ctor {
    crate::ir::Ctor::StructlikeCtor(sctor) => {
      let mut restricted = HashMap::new();
      for inst in sctor.insts(db) {
        restricted.insert(inst.id(db), id2sym.inst2sym(db)[&inst.id(db)].clone());
      }
      irlf_ser::ir::Ctor::StructlikeCtor(irlf_ser::ir::StructlikeCtor {
        inst2sym: restricted,
        insts: sctor
          .insts(db)
          .iter()
          .map(|inst| (inst.id(db), unconvert_inst(db, inst)))
          .collect(),
        left: sctor.left(db).iter().map(|iface| iface.id(db)).collect(),
        right: sctor.right(db).iter().map(|iface| iface.id(db)).collect(),
        connections: sctor
          .connections(db)
          .iter()
          .map(|c| unconvert_connection(db, c))
          .collect(),
      })
    }
    crate::ir::Ctor::BinaryCtor(bctor) => {
      irlf_ser::ir::Ctor::BinaryCtor(irlf_ser::ir::BinaryCtor {
        path: bctor.path(db),
      })
    }
    crate::ir::Ctor::LibCtor(lctor) => irlf_ser::ir::Ctor::LibCtor(irlf_ser::ir::LibCtor {
      name: lctor.name(db).clone(),
    }),
  }
}

fn unconvert_inst(db: &dyn Db, inst: &crate::ir::Inst) -> irlf_ser::ir::CtorCall {
  irlf_ser::ir::CtorCall {
    ctor: inst.ctor(db).id(db),
  }
}

fn unconvert_instref(db: &dyn Db, iref: &crate::ir::InstRef) -> irlf_ser::ir::InstRef {
  irlf_ser::ir::InstRef(iref.iref(db).iter().map(|inst| inst.id(db)).collect())
}

fn unconvert_connection(db: &dyn Db, c: &crate::ir::Connection) -> irlf_ser::ir::Connection {
  irlf_ser::ir::Connection {
    id: c.id(db),
    left: unconvert_instref(db, c.left(db)),
    right: unconvert_instref(db, c.right(db)),
  }
}

#[cfg(test)]
mod test {
  use pretty_assertions::assert_eq;

  use super::*;

  #[derive(Default)]
  #[salsa::db(crate::Jar)]
  pub(crate) struct TestDatabase {
    storage: salsa::Storage<Self>,
  }

  impl salsa::Database for TestDatabase {}

  #[test]
  fn test_convert() {
    let text = "cmxy 0x99 times2
---
a 0x1 /this/is/a/path
b 0x2 /this/is/another/path
---
rtor0 0x3
  foo 89 = 0x2
  ---
  ---
  89
  ---
  90 89 89
rtor1 0x4
  baz 87 = 0x2
  bar 88 = 0x3
  ---
  87 88
  ---
  88 87
  ---
  91 88 87
  92 87 87
---
0x3
";
    let source = irlf_ser::unpretty::unpretty(text).unwrap();
    let db = TestDatabase::default();
    let source = crate::ir::SourceProgram::new(&db, source);
    let (program, id2sym) = crate::convert::convert(&db, source);
    let round_tripped = unconvert(&db, program, id2sym);
    let actual = format!("{round_tripped}");
    assert_eq!(text, actual);
  }
}
