use std::collections::HashMap;

use irlf_ser::ir::{CtorId, InstId, Sym};

use crate::visitor::Visitor;

pub struct UnBuilder(irlf_ser::ir::Program);

impl UnBuilder {
  fn new(p: &crate::ir::Program, ctorid2sym: HashMap<CtorId, Sym>) -> UnBuilder {
    UnBuilder(irlf_ser::ir::Program {
      ctors: HashMap::new(),
      ctorid2sym,
      main: p.main.id,
    })
  }
}

fn unbuild_iface(iface: &crate::ir::Iface) -> irlf_ser::ir::Iface {
  iface.iter().map(|it| it.id).collect()
}

fn unbuild_instref(instref: &crate::ir::InstRef) -> irlf_ser::ir::InstRef {
  irlf_ser::ir::InstRef(instref.0.iter().map(|call| call.id).collect())
}

fn unbuild_insts(body: &crate::ir::StructlikeCtorBody) -> HashMap<InstId, irlf_ser::ir::CtorCall> {
  let mut insts = HashMap::new();
  for (instid, call) in body.insts.iter().map(|it| (it.id, it.ctor)) {
    insts.insert(instid, irlf_ser::ir::CtorCall { ctor: call.id });
  }
  insts
}

impl<'a> Visitor<'a> for UnBuilder {
  fn binary_ctor(&mut self, id: CtorId, ctor: &crate::ir::BinaryCtor) {
    self.0.ctors.insert(
      id,
      irlf_ser::ir::Ctor::BinaryCtor(irlf_ser::ir::BinaryCtor {
        path: ctor.path.clone(),
      }),
    );
  }
  fn structlike_ctor(&mut self, id: CtorId, sctor: &crate::ir::StructlikeCtor<'a>) {
    let body = sctor.body.get().unwrap();
    let bodybody = body.body.get().unwrap();
    self.0.ctors.insert(
      id,
      irlf_ser::ir::Ctor::StructlikeCtor(irlf_ser::ir::StructlikeCtor {
        inst2sym: HashMap::new(),
        insts: unbuild_insts(body),
        left: unbuild_iface(&bodybody.left),
        right: unbuild_iface(&bodybody.right),
        connections: bodybody
          .connections
          .iter()
          .map(|it| irlf_ser::ir::Connection {
            id: it.id,
            left: unbuild_instref(&it.left),
            right: unbuild_instref(&it.right),
          })
          .collect(),
      }),
    );
  }
}

#[must_use]
pub fn unbuild(p: &crate::ir::Program, ctorid2sym: HashMap<CtorId, Sym>) -> irlf_ser::ir::Program {
  let mut b = UnBuilder::new(p, ctorid2sym);
  b.program(p);
  b.0
}
