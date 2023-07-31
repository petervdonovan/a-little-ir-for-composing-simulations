use std::cell::OnceCell;
use std::collections::HashMap;

use crate::ir::{Arenas, StructlikeCtorBody};

macro_rules! make_builder {
  ($builder_name: ident, $successor_name: ident) => {
    pub struct $builder_name<'a> {
      arenas: &'a Arenas<'a>,
      source: irlf_ser::ir::Program,
      ctorid2ctor: HashMap<irlf_ser::ir::CtorId, &'a crate::ir::Ctor<'a>>,
      instid2inst: HashMap<irlf_ser::ir::InstId, &'a crate::ir::CtorCall<'a>>,
      program: crate::ir::Program<'a>,
    }
    impl<'a> $builder_name<'a> {
      fn successor(self) -> $successor_name<'a> {
        $successor_name {
          arenas: self.arenas,
          source: self.source,
          ctorid2ctor: self.ctorid2ctor,
          instid2inst: self.instid2inst,
          program: self.program,
        }
      }
    }
  };
}

make_builder!(Builder0, Builder1);
make_builder!(Builder1, Builder2);
make_builder!(Builder2, Builder2);

impl<'a> irlf_ser::visitor::Visitor for Builder0<'a> {
  fn binary_ctor(&mut self, id: irlf_ser::ir::CtorId, binary_ctor: &irlf_ser::ir::BinaryCtor) {
    let binary_ctor = self.arenas.binary_ctors_arena.alloc(crate::ir::BinaryCtor {
      path: binary_ctor.path.clone(),
    });
    let ctor = self.arenas.ctors_arena.alloc(crate::ir::Ctor {
      id,
      imp: crate::ir::CtorImpl::BinaryCtor(binary_ctor),
    });
    self.program.ctors.push(ctor);
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
    self.program.ctors.push(ctor);
  }
}

impl<'a> crate::visitor::Visitor<'a> for Builder1<'a> {
  fn structlike_ctor(&mut self, id: irlf_ser::ir::CtorId, sctor: &crate::ir::StructlikeCtor<'a>) {
    let source: &irlf_ser::ir::Ctor = self.source.ctors.get(&id).unwrap();
    let mut insts = vec![];
    if let irlf_ser::ir::Ctor::StructlikeCtor(source) = source {
      for (id, inst) in &source.insts {
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

impl<'a> crate::visitor::Visitor<'a> for Builder2<'a> {
  fn children_structlike_ctor_body(
    &mut self,
    id: irlf_ser::ir::CtorId,
    _sctor: &crate::ir::StructlikeCtor,
    body: &StructlikeCtorBody<'a>,
  ) {
    let source = self.source.ctors.get(&id).unwrap();
    let mut connections = vec![];
    if let irlf_ser::ir::Ctor::StructlikeCtor(source) = source {
      for connection in &source.connections {
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
      body
        .body
        .set(crate::ir::StructlikeCtorBodyBody { connections })
        .unwrap();
    } else {
      panic!();
    }
  }
}

impl<'a> Builder2<'a> {
  fn finish(self) -> crate::ir::Program<'a> {
    self.program
  }
}
