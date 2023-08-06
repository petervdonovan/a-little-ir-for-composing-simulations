use irlf_db::ir::{Ctor, StructlikeCtor};

use crate::{
  rtor::{InputsGiver, InputsIfaceGiver, Rtor, RtorIface},
  Db,
};

use super::iface_of;

pub struct Srtor<'db> {
  downstream: Option<InputsGiver<'db>>,
}

pub struct SrtorIface<'db> {
  downstream: Option<InputsIfaceGiver<'db>>,
  ctor: StructlikeCtor,
  children: Vec<Box<dyn RtorIface<'db>>>,
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

impl<'db> SrtorIface<'db> {
  pub fn new(db: &dyn irlf_db::Db, sctor: &StructlikeCtor) -> Self {
    let children = sctor
      .insts(db)
      .iter()
      .map(|inst| iface_of(db, inst.ctor(db)))
      .collect();
    SrtorIface {
      downstream: None,
      ctor: sctor.clone(),
      children,
    }
  }
}

impl<'db> RtorIface<'db> for SrtorIface<'db> {
  fn accept(&mut self, side: lf_types::Side, inputs: crate::rtor::InputsIfaceGiver<'db>) {
    todo!()
  }

  fn provide(&'db self, side: lf_types::Side) -> crate::rtor::InputsIfaceGiver<'db> {
    todo!()
  }

  fn iterate_levels(&mut self) -> bool {
    todo!()
  }

  fn levels(&self) -> Vec<u32> {
    todo!()
  }

  fn realize(&self, _inst_time_args: Vec<&'db dyn std::any::Any>) -> Box<dyn Rtor> {
    todo!()
  }
}
