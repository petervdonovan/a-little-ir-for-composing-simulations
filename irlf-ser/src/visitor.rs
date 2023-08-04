use crate::ir::{
  BinaryCtor, Connection, Ctor, CtorCall, CtorId, InstId, LibCtor, Program, StructlikeCtor,
};

pub trait Visitor {
  fn program(&mut self, p: &Program) {
    self.children_program(p);
  }

  fn binary_ctor(&mut self, _: CtorId, _: &BinaryCtor) {}

  fn structlike_ctor(&mut self, id: CtorId, sctor: &StructlikeCtor) {
    self.children_structlike_ctor(id, sctor);
  }

  fn lib_ctor(&mut self, _: CtorId, _: &LibCtor) {}

  fn main(&mut self, _id: CtorId) {}

  fn instid_sym(&mut self, _id: InstId, _sym: &str) {}

  fn ctorid_sym(&mut self, _ctorid: CtorId, _sym: &str) {}

  fn inst(&mut self, _parent: &StructlikeCtor, _id: InstId, _inst: &CtorCall) {}

  fn connection(&mut self, _parent: &StructlikeCtor, _connection: &Connection) {}

  fn children_program(&mut self, p: &Program) {
    let Program {
      ctorid2sym,
      ctors,
      main,
    } = p;
    for (id, sym) in ctorid2sym {
      self.ctorid_sym(*id, sym);
    }
    for (id, ctor) in ctors {
      match ctor {
        Ctor::BinaryCtor(bctor) => self.binary_ctor(*id, bctor),
        Ctor::StructlikeCtor(sctor) => self.structlike_ctor(*id, sctor),
        Ctor::LibCtor(lctor) => self.lib_ctor(*id, lctor),
      }
    }
    self.main(*main);
  }
  fn children_structlike_ctor(&mut self, _: CtorId, ctor: &StructlikeCtor) {
    let StructlikeCtor {
      inst2sym,
      insts,
      left: _,
      right: _,
      connections,
    } = ctor;
    for (id, sym) in inst2sym {
      self.instid_sym(*id, sym);
    }
    for (id, ctorcall) in insts {
      self.inst(ctor, *id, ctorcall);
    }
    for connection in connections {
      self.connection(ctor, connection);
    }
  }
}
