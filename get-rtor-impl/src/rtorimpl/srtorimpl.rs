use irlf_db::ir::StructlikeCtor;

use crate::rtor::{InputsGiver, InputsIface, Rtor, RtorIface};

use super::iface_of;

pub struct Srtor<'db> {
  downstream: Option<InputsGiver<'db>>,
}

pub struct SrtorIface<'a> {
  downstream: Option<InputsIface<'a>>,
  ctor: StructlikeCtor,
  children: Vec<Box<dyn RtorIface<'a> + 'a>>,
}

impl<'db> Rtor<'db> for Srtor<'db> {
  fn accept(&mut self, side: lf_types::Side, inputs: crate::rtor::InputsGiver<'db>) -> bool {
    todo!()
  }

  fn provide(&'db self, side: lf_types::Side) -> crate::rtor::InputsGiver<'db> {
    todo!()
  }

  fn step_forward(&mut self, distance: u64) -> Option<lf_types::Net> {
    todo!()
  }

  fn step_down(&mut self) {
    todo!()
  }

  fn step_up(&mut self) -> Option<lf_types::Net> {
    todo!()
  }
}

impl<'a> SrtorIface<'a> {
  pub fn new(db: &'a dyn irlf_db::Db, sctor: StructlikeCtor) -> Self {
    let children: Vec<Box<dyn RtorIface<'a> + 'a>> = sctor
      .insts(db)
      .iter()
      .map(|inst| iface_of(db, inst.ctor(db)))
      .collect();
    SrtorIface {
      downstream: None,
      ctor: sctor,
      children,
    }
  }
}

impl<'a> RtorIface<'a> for SrtorIface<'a> {
  fn accept(&'a mut self, side: lf_types::Side, inputs: crate::rtor::InputsIface<'a>) {
    todo!()
  }

  fn provide(&'a self, side: lf_types::Side) -> crate::rtor::InputsIface<'a> {
    todo!()
  }

  fn iterate_levels(&mut self) -> bool {
    todo!()
  }

  fn levels(&self) -> Vec<u32> {
    todo!()
  }

  fn realize<'db>(&self, _inst_time_args: Vec<&'db dyn std::any::Any>) -> Box<dyn Rtor<'db> + 'db> {
    todo!()
  }
}
