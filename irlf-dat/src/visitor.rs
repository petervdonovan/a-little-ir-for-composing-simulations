use irlf_ser::ir::CtorId;

use crate::ir::{
  BinaryCtor, Connection, Ctor, CtorCall, CtorImpl, Program, StructlikeCtor, StructlikeCtorBody,
  StructlikeCtorBodyBody,
};

pub trait Visitor<'a> {
  fn program(&mut self, p: &Program<'a>) {
    self.children_program(p);
  }
  fn ctor(&mut self, ctor: &Ctor<'a>) {
    self.children_ctor(ctor);
  }
  fn main(&mut self, _ctor: &Ctor<'a>) {}
  fn ctor_impl(&mut self, id: CtorId, imp: &CtorImpl<'a>) {
    self.children_ctor_impl(id, imp);
  }
  fn structlike_ctor(&mut self, id: CtorId, sctor: &StructlikeCtor<'a>) {
    self.children_structlike_ctor(id, sctor);
  }
  fn binary_ctor(&mut self, _id: CtorId, _ctor: &BinaryCtor) {}
  fn inst(
    &mut self,
    _parent_id: CtorId,
    _parent: &StructlikeCtor,
    _parent_body: &StructlikeCtorBody,
    _: &CtorCall,
  ) {
  }
  fn structlike_ctor_body(&mut self, id: CtorId, sctor: &StructlikeCtorBody) {}
  fn structlike_ctor_body_body(&mut self, id: CtorId, sctor: &StructlikeCtorBodyBody<'a>) {
    self.children_structlike_ctor_body_body(id, sctor);
  }
  fn connection(&mut self, _parent: CtorId, _connection: &Connection<'a>) {}
  fn children_program(&mut self, p: &Program<'a>) {
    let Program { ctors, main } = p;
    for ctor in ctors {
      self.ctor(ctor);
    }
    self.main(main);
  }
  fn children_ctor(&mut self, ctor: &Ctor<'a>) {
    let Ctor { id, imp } = ctor;
    self.ctor_impl(*id, imp);
  }
  fn children_ctor_impl(&mut self, id: CtorId, imp: &CtorImpl<'a>) {
    match imp {
      CtorImpl::StructlikeCtor(sctor) => self.structlike_ctor(id, sctor),
      CtorImpl::BinaryCtor(bctor) => self.binary_ctor(id, bctor),
    }
  }
  fn children_structlike_ctor(&mut self, id: CtorId, sctor: &StructlikeCtor<'a>) {
    let StructlikeCtor { body } = sctor;
    if let Some(body) = body.get() {
      self.structlike_ctor_body(id, body);
    }
  }
  fn children_structlike_ctor_body(
    &mut self,
    id: CtorId,
    sctor: &StructlikeCtor,
    body: &StructlikeCtorBody<'a>,
  ) {
    let StructlikeCtorBody {
      insts,
      body: bodybody,
    } = body;
    for inst in insts {
      self.inst(id, sctor, body, inst);
    }
    if let Some(body) = bodybody.get() {
      self.structlike_ctor_body_body(id, body);
    }
  }
  fn children_structlike_ctor_body_body(&mut self, id: CtorId, sctor: &StructlikeCtorBodyBody<'a>) {
    let StructlikeCtorBodyBody { connections } = sctor;
    for connection in connections {
      self.connection(id, connection);
    }
  }
}
