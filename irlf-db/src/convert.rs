use std::{cell::RefCell, collections::HashMap};

use irlf_ser::ir::{CtorId, InstId};

use crate::Db;

#[salsa::tracked]
pub fn convert(db: &dyn crate::Db, source: crate::ir::SourceProgram) -> crate::ir::Program {
  let ctors = vec![];
  let mut ctorid2ctor = HashMap::new();
  for (id, ctor) in source.source(db).ctors.iter() {
    ctorid2ctor.insert(id, convert_ctor(db, &source.source(db).ctors, *id, ctor));
  }
  let main = ctorid2ctor[&source.source(db).main];
  crate::ir::Program::new(db, ctors, main)
}

std::thread_local! {
  static MEMO_CTORS : RefCell<HashMap<CtorId, crate::ir::Ctor>> = RefCell::new(HashMap::new());
  static MEMO_INSTS : RefCell<HashMap<InstId, crate::ir::Inst>> = RefCell::new(HashMap::new());
}

macro_rules! check_cache {
  ($cache_name: ident, $id: ident) => {
    if let Some(ret) = $cache_name.with(|map| {
      let borrowed = map.borrow_mut();
      borrowed.get(&$id).cloned()
    }) {
      return ret;
    }
  };
}

fn convert_ctor(
  db: &dyn Db,
  ctorid2ctor: &HashMap<CtorId, irlf_ser::ir::Ctor>,
  id: CtorId,
  ctor: &irlf_ser::ir::Ctor,
) -> crate::ir::Ctor {
  check_cache!(MEMO_CTORS, id);
  match ctor {
    irlf_ser::ir::Ctor::StructlikeCtor(sctor) => {
      let insts = convert_insts(db, ctorid2ctor, sctor);
      let left = convert_iface(db, ctorid2ctor, &sctor.insts, &sctor.left);
      let right = convert_iface(db, ctorid2ctor, &sctor.insts, &sctor.right);
      let connections = convert_connections(db, ctorid2ctor, &sctor.insts, &sctor.connections);
      crate::ir::Ctor::StructlikeCtor(crate::ir::StructlikeCtor::new(
        db,
        id,
        insts,
        left,
        right,
        connections,
      ))
    }
    irlf_ser::ir::Ctor::BinaryCtor(bctor) => {
      crate::ir::Ctor::BinaryCtor(crate::ir::BinaryCtor::new(db, id, bctor.path.clone()))
    }
  }
}

fn convert_connections(
  db: &dyn Db,
  ctorid2ctor: &HashMap<CtorId, irlf_ser::ir::Ctor>,
  instid2inst: &HashMap<InstId, irlf_ser::ir::CtorCall>,
  connections: &[irlf_ser::ir::Connection],
) -> Vec<crate::ir::Connection> {
  connections
    .iter()
    .map(|c| {
      crate::ir::Connection::new(
        db,
        c.id,
        convert_instref(db, ctorid2ctor, instid2inst, &c.left),
        convert_instref(db, ctorid2ctor, instid2inst, &c.right),
      )
    })
    .collect()
}

fn convert_instref(
  db: &dyn Db,
  ctorid2ctor: &HashMap<CtorId, irlf_ser::ir::Ctor>,
  instid2inst: &HashMap<InstId, irlf_ser::ir::CtorCall>,
  iref: &irlf_ser::ir::InstRef,
) -> crate::ir::InstRef {
  crate::ir::InstRef::new(
    db,
    iref
      .0
      .iter()
      .map(|id| convert_ctorcall(db, ctorid2ctor, *id, &instid2inst[id]))
      .collect(),
  )
}

fn convert_iface(
  db: &dyn Db,
  ctorid2ctor: &HashMap<CtorId, irlf_ser::ir::Ctor>,
  instid2inst: &HashMap<InstId, irlf_ser::ir::CtorCall>,
  iface: &irlf_ser::ir::Iface,
) -> crate::ir::Iface {
  iface
    .iter()
    .map(|id| convert_ctorcall(db, ctorid2ctor, *id, &instid2inst[id]))
    .collect()
}

fn convert_insts(
  db: &dyn Db,
  ctorid2ctor: &HashMap<CtorId, irlf_ser::ir::Ctor>,
  sctor: &irlf_ser::ir::StructlikeCtor,
) -> Vec<crate::ir::Inst> {
  sctor
    .insts
    .iter()
    .map(|(id, i)| convert_ctorcall(db, ctorid2ctor, *id, i))
    .collect()
}

fn convert_ctorcall(
  db: &dyn Db,
  ctorid2ctor: &HashMap<CtorId, irlf_ser::ir::Ctor>,
  id: InstId,
  call: &irlf_ser::ir::CtorCall,
) -> crate::ir::Inst {
  check_cache!(MEMO_INSTS, id);
  crate::ir::Inst::new(
    db,
    id,
    convert_ctor(db, ctorid2ctor, call.ctor, &ctorid2ctor[&call.ctor]),
  )
}
