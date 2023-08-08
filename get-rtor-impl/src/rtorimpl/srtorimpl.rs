use std::collections::HashMap;

use irlf_db::{
  ir::{Inst, StructlikeCtor},
  Db,
};
use lf_types::Side;

use crate::{
  iterators::cloneiterator::CloneIterator,
  rtor::{InputsGiver, InputsIface, Rtor, RtorIface, ShareLevelLowerBound},
};

use super::iface_of;
use crate::iterators::chainclone::ChainClone;

pub struct Srtor<'db> {
  downstream: Option<InputsGiver<'db>>,
}

pub struct SrtorIface<'a> {
  db: &'a dyn Db,
  ldownstream: Option<InputsIface>,
  rdownstream: Option<InputsIface>,
  ctor: StructlikeCtor,
  children: HashMap<Inst, Box<dyn RtorIface<'a> + 'a>>,
  levels_internal2external: HashMap<u32, u32>,
}

struct SideIterator<'a> {
  ctor: StructlikeCtor,
  db: &'a dyn Db,
  side: Side,
  pos: usize,
}

impl<'a> SideIterator<'a> {
  fn new(db: &'a dyn Db, ctor: StructlikeCtor, side: Side) -> Self {
    SideIterator {
      ctor,
      db,
      side,
      pos: 0,
    }
  }
}

impl<'a> Iterator for SideIterator<'a> {
  type Item = Inst;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let current = self.ctor.iface(self.db).get(self.pos)?;
      if self.side == current.0 {
        return Some(current.1);
      }
    }
  }
}

impl<'a> SrtorIface<'a> {
  fn downstream(&mut self, side: Side) -> &mut Option<InputsIface> {
    match side {
      Side::Left => &mut self.ldownstream,
      Side::Right => &mut self.rdownstream,
    }
  }
}

fn iface(sctor: StructlikeCtor, db: &dyn Db, side: Side) -> SideIterator {
  SideIterator::new(db, sctor, side)
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
    let children = sctor
      .insts(db)
      .iter()
      .map(|inst| (*inst, iface_of(db, inst.ctor(db))))
      .collect();
    SrtorIface {
      db,
      ldownstream: None,
      rdownstream: None,
      ctor: sctor,
      children,
      levels_internal2external: HashMap::new(),
    };
    todo!("initialize levels_internal2external?")
  }
}

impl<'a> RtorIface<'a> for SrtorIface<'a> {
  fn accept(&mut self, part: &[Inst], side: lf_types::Side, inputs: &mut crate::rtor::InputsIface) {
    // FIXME: this is all wrong
    *self.downstream(side) = Some(inputs.clone());
    for child in iface(self.ctor, self.db, side) {
      self
        .children
        .get_mut(&child)
        .unwrap()
        .accept(&part[1..], side, inputs);
    }
  }

  fn provide(&'a self, part: &[Inst], side: lf_types::Side) -> crate::rtor::InputsIface {
    // FIXME: this is all wrong
    let mut sub_iterators = vec![];
    for child in iface(self.ctor, self.db, side) {
      sub_iterators.push(self.children[&child].provide(&part[1..], side));
    }
    Box::new(ChainClone::new(sub_iterators))
  }

  fn iterate_levels(&mut self) -> bool {
    // FIXME: this is all wrong
    let mut changed = false;
    for (_, child) in self.children.iter_mut() {
      changed |= child.iterate_levels();
    }
    changed
  }

  fn levels(&self) -> Vec<u32> {
    todo!()
  }

  fn in_levels(&self) -> Box<dyn crate::rtor::LevelIterator> {
    todo!()
  }

  fn out_levels(&self) -> Box<dyn crate::rtor::LevelIterator> {
    todo!()
  }

  fn realize<'db>(&self, _inst_time_args: Vec<&'db dyn std::any::Any>) -> Box<dyn Rtor<'db> + 'db> {
    todo!()
  }
}
