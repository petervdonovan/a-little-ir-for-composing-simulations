use crate::ir_serializable::{
  BinaryCtor, Connection, Ctor, CtorCall, CtorId, InstId, Program, StructlikeCtor,
};

pub trait Visitor {
  fn program(&mut self, p: &Program) {
    self.children_program(p);
  }

  fn binary_ctor(&mut self, _: &CtorId, _: &BinaryCtor) {}

  fn structlike_ctor(&mut self, id: &CtorId, sctor: &StructlikeCtor) {
    self.children_structlike_ctor(id, sctor);
  }

  fn main(&mut self, _: &CtorId) {}

  fn instid_sym(&mut self, _: &InstId, _: &String) {}

  fn ctorid_sym(&mut self, _: &CtorId, _: &String) {}

  fn inst(&mut self, _: &InstId, _: &CtorCall) {}

  fn connection(&mut self, _: &Connection) {}

  fn children_program(&mut self, p: &Program) {
    let Program {
      ctorid2sym,
      ctors,
      main,
    } = p;
    for (id, sym) in ctorid2sym {
      self.ctorid_sym(id, sym);
    }
    for (id, ctor) in ctors {
      match ctor {
        Ctor::BinaryCtor(bctor) => self.binary_ctor(id, bctor),
        Ctor::StructlikeCtor(sctor) => self.structlike_ctor(id, sctor),
      }
    }
    self.main(main);
  }
  fn children_structlike_ctor(&mut self, _: &CtorId, ctor: &StructlikeCtor) {
    let StructlikeCtor {
      inst2sym,
      insts,
      left: _,
      right: _,
      connections,
    } = ctor;
    for (id, sym) in inst2sym {
      self.instid_sym(id, sym);
    }
    for (id, ctorcall) in insts {
      self.inst(id, ctorcall);
    }
    for connection in connections {
      self.connection(connection);
    }
  }
}
