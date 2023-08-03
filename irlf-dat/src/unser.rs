use std::cell::OnceCell;
use std::collections::HashMap;

use irlf_ser::{
  ir::{CtorId, InstId},
  visitor::Visitor,
};

use crate::ir::{Arenas, CtorCall, StructlikeCtorBody};

pub struct Builder0<'a, 'b> {
  arenas: &'a Arenas<'a>,
  source: &'b irlf_ser::ir::Program,
  ctorid2ctor: HashMap<irlf_ser::ir::CtorId, &'a crate::ir::Ctor<'a>>,
  instid2inst: HashMap<irlf_ser::ir::InstId, &'a crate::ir::CtorCall<'a>>,
  ctors: Vec<&'a crate::ir::Ctor<'a>>,
}

impl<'a, 'b> Builder0<'a, 'b> {
  fn successor(self) -> Builder1<'a> {
    let main = *self.ctorid2ctor.get(&self.source.main).unwrap();
    Builder1 {
      arenas: self.arenas,
      ctorid2ctor: self.ctorid2ctor,
      instid2inst: self.instid2inst,
      program: crate::ir::Program {
        ctors: self.ctors,
        main,
      },
    }
  }
}

pub struct Builder1<'a> {
  arenas: &'a Arenas<'a>,
  ctorid2ctor: HashMap<irlf_ser::ir::CtorId, &'a crate::ir::Ctor<'a>>,
  instid2inst: HashMap<irlf_ser::ir::InstId, &'a crate::ir::CtorCall<'a>>,
  program: crate::ir::Program<'a>,
}
impl<'a> Builder1<'a> {
  fn successor(self) -> Builder2<'a> {
    Builder2 {
      ctorid2ctor: self.ctorid2ctor,
      instid2inst: self.instid2inst,
      program: self.program,
    }
  }
}
pub struct Builder2<'a> {
  ctorid2ctor: HashMap<irlf_ser::ir::CtorId, &'a crate::ir::Ctor<'a>>,
  instid2inst: HashMap<irlf_ser::ir::InstId, &'a crate::ir::CtorCall<'a>>,
  program: crate::ir::Program<'a>,
}

impl<'a, 'b> irlf_ser::visitor::Visitor for Builder0<'a, 'b> {
  fn binary_ctor(&mut self, id: irlf_ser::ir::CtorId, binary_ctor: &irlf_ser::ir::BinaryCtor) {
    let binary_ctor = self.arenas.binary_ctors_arena.alloc(crate::ir::BinaryCtor {
      path: binary_ctor.path.clone(),
    });
    let ctor = self.arenas.ctors_arena.alloc(crate::ir::Ctor {
      id,
      imp: crate::ir::CtorImpl::BinaryCtor(binary_ctor),
    });
    self.ctors.push(ctor);
    self.ctorid2ctor.insert(id, ctor);
  }
  fn structlike_ctor(&mut self, id: irlf_ser::ir::CtorId, _: &irlf_ser::ir::StructlikeCtor) {
    let structlike_ctor = self
      .arenas
      .structlike_ctors_arena
      .alloc(crate::ir::StructlikeCtor {
        body: OnceCell::new(),
      });
    let ctor = self.arenas.ctors_arena.alloc(crate::ir::Ctor {
      id,
      imp: crate::ir::CtorImpl::StructlikeCtor(structlike_ctor),
    });
    self.ctors.push(ctor);
    self.ctorid2ctor.insert(id, ctor);
  }
}

impl<'a> irlf_ser::visitor::Visitor for Builder1<'a> {
  fn structlike_ctor(&mut self, id: CtorId, source_sctor: &irlf_ser::ir::StructlikeCtor) {
    let new_sctor = *self.ctorid2ctor.get(&id).unwrap();
    let mut insts = vec![];
    if let crate::ir::CtorImpl::StructlikeCtor(sctor) = new_sctor.imp {
      for (id, inst) in &source_sctor.insts {
        let inst: &crate::ir::CtorCall<'_> = self.arenas.insts_arena.alloc(crate::ir::CtorCall {
          id: *id,
          ctor: self.ctorid2ctor.get(&inst.ctor).unwrap(),
        });
        insts.push(inst);
        self.instid2inst.insert(*id, inst);
      }
      sctor
        .body
        .set(StructlikeCtorBody {
          insts,
          body: OnceCell::new(),
        })
        .unwrap();
    } else {
      panic!()
    }
  }
}

impl<'a> Builder2<'a> {
  fn convert_connections(
    &self,
    source_sctor: &irlf_ser::ir::StructlikeCtor,
  ) -> Vec<crate::ir::Connection<'a>> {
    let mut connections = vec![];
    for connection in &source_sctor.connections {
      let mut left = vec![];
      let mut right = vec![];
      for instid in &connection.left.0 {
        left.push(*self.instid2inst.get(instid).unwrap());
      }
      for instid in &connection.right.0 {
        right.push(*self.instid2inst.get(instid).unwrap());
      }
      connections.push(crate::ir::Connection {
        id: connection.id,
        left: crate::ir::InstRef(left),
        right: crate::ir::InstRef(right),
      });
    }
    connections
  }
  fn convert_iface(&self, source_iface: &Vec<InstId>) -> Vec<&'a CtorCall<'a>> {
    let mut new_iface = vec![];
    for instid in source_iface {
      let inst = *self.instid2inst.get(instid).unwrap();
      new_iface.push(inst);
    }
    new_iface
  }
}

impl<'a> irlf_ser::visitor::Visitor for Builder2<'a> {
  fn structlike_ctor(&mut self, id: CtorId, source_sctor: &irlf_ser::ir::StructlikeCtor) {
    let new_sctor = self.ctorid2ctor.get(&id).unwrap();
    if let crate::ir::CtorImpl::StructlikeCtor(new_sctor) = new_sctor.imp {
      let connections = self.convert_connections(source_sctor);
      let left = self.convert_iface(&source_sctor.left);
      let right = self.convert_iface(&source_sctor.right);
      new_sctor
        .body
        .get()
        .unwrap()
        .body
        .set(crate::ir::StructlikeCtorBodyBody {
          left,
          right,
          connections,
        })
        .unwrap();
    } else {
      panic!();
    }
  }
}

impl<'a, 'b> Builder0<'a, 'b> {
  fn new(arenas: &'a Arenas<'a>, p: &'b irlf_ser::ir::Program) -> Builder0<'a, 'b> {
    Builder0 {
      arenas,
      source: p,
      ctorid2ctor: HashMap::new(),
      instid2inst: HashMap::new(),
      ctors: vec![],
    }
  }
}

impl<'a> Builder2<'a> {
  fn finish(self) -> crate::ir::Program<'a> {
    self.program
  }
}

pub fn build<'a>(arenas: &'a Arenas<'a>, p: &irlf_ser::ir::Program) -> crate::ir::Program<'a> {
  let mut b = Builder0::new(arenas, p);
  b.program(p);
  let mut b = b.successor();
  b.program(p);
  let mut b = b.successor();
  b.program(p);
  b.finish()
}

mod test {
  use super::*;
  use irlf_ser::unpretty::unpretty;

  #[test]
  fn test_program() {
    let p = unpretty(
      "a 0x1 /this/is/a/path
b 0x2 /this/is/another/path
---
rtor0 0x3
  foo 89 = 0x4
  ---
  ---
  89
  ---
  90 89 89
rtor1 0x4
  baz 87 = 0x3
  bar 88 = 0x4
  ---
  87 88
  ---
  88 88
  ---
  91 88 87
  92 87 87
---
0x3
",
    )
    .unwrap();
    println!("{p:?}");
    let arenas = Arenas::new();
    let p = build(&arenas, &p);
    println!("{p:?}");
  }
}
