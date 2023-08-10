use std::{collections::HashMap, hash::Hash, rc::Rc};

use irlf_db::{
  ir::{Connection, Inst, StructlikeCtor},
  Db,
};
use lf_types::{Level, Side};

use crate::{
  iterators::cloneiterator::CloneIterator,
  rtor::{
    InputsGiver, InputsIface, LevelIterator, Rtor, RtorComptime, RtorIface, ShareLevelLowerBound,
  },
};

use super::iface_of;
use crate::iterators::chainclone::ChainClone;

// dyn_clone::clone_trait_object!(ChainClone<Level, dyn LevelIterator<Item = Level>>);
// impl CloneIterator<Level> for ChainClone<Level, Box<dyn CloneIterator<Level>>> {}

pub struct Srtor<'db> {
  downstream: Option<InputsGiver<'db>>,
}

pub struct SrtorComptime {
  ldownstream: Option<InputsIface>,
  rdownstream: Option<InputsIface>,
  levels_internal2external: HashMap<Level, Level>,
  sctor: StructlikeCtor,
}

pub struct SrtorIface<'a> {
  db: &'a dyn Db,
  sctor: StructlikeCtor,
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

impl SrtorComptime {
  fn new(sctor: StructlikeCtor) -> Self {
    SrtorComptime {
      ldownstream: None,
      rdownstream: None,
      levels_internal2external: HashMap::new(),
      sctor,
    }
  }
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

fn connect<'db>(
  db: &dyn Db,
  children: &mut HashMap<Inst, Box<dyn RtorComptime + 'db>>,
  sctor: StructlikeCtor,
) {
  let mut connect =
    |ref_accept: Vec<Inst>, ref_provide: Vec<Inst>, side_accept: Side, side_provide: Side| {
      let mut provide1 = children[&ref_provide[0]].provide(&ref_provide[1..], side_accept);
      children.get_mut(&ref_accept[0]).unwrap().accept(
        &ref_accept[1..],
        side_provide,
        &mut provide1,
      )
    };
  for connection in sctor.connections(db) {
    let lref = connection.left(db).iref(db);
    let rref = connection.right(db).iref(db);
    connect(lref, rref, Side::Left, Side::Right);
  }
}

fn fixpoint<'db>(children: &mut HashMap<Inst, Box<dyn RtorComptime + 'db>>) {
  let mut changed = false;
  for (_, child) in children.iter_mut() {
    changed |= child.iterate_levels();
  }
  if changed {
    fixpoint(children);
  }
  // todo: what if the children arrive at a level assignment that is incompatible with the ambient
  // TPO? You must add more constraints
}

impl<'a> SrtorIface<'a> {
  pub fn new(db: &'a dyn irlf_db::Db, sctor: StructlikeCtor) -> Self {
    SrtorIface { db, sctor }
  }
  fn side(&self, side: Side) -> Box<dyn Iterator<Item = Box<dyn RtorIface + 'a>> + 'a> {
    Box::new(iface(self.sctor, self.db, side).map(|child| iface_of(self.db, child.ctor(self.db))))
  }
}

impl RtorComptime for SrtorComptime {
  fn iterate_levels(&mut self) -> bool {
    todo!()
  }

  fn levels(&self) -> Vec<Level> {
    todo!()
  }
  fn accept(&mut self, part: &[Inst], side: lf_types::Side, inputs: &mut crate::rtor::InputsIface) {
    // FIXME: this is all wrong
    // *self.downstream(side) = Some(inputs.clone());
    // for child in iface(self.ctor, self.db, side) {
    //   self
    //     .children
    //     .get_mut(&child)
    //     .unwrap()
    //     .accept(&part[1..], side, inputs);
    // }
    todo!()
  }

  fn provide(&self, part: &[Inst], side: lf_types::Side) -> crate::rtor::InputsIface {
    todo!()
  }
}

impl<'a> RtorIface for SrtorIface<'a> {
  fn immut_accept(
    &self,
    part: &[Inst],
    side: Side,
    provided_levels: &mut LevelIterator,
    f: Rc<dyn Fn(Level, Level)>,
  ) {
    for iface in self.side(side) {
      match part {
        [hd, tail @ ..] => {
          todo!()
        }
        [] => {
          for iface in self.side(side) {
            iface.immut_accept(part, side, provided_levels, Rc::clone(&f));
          }
        }
      }
    }
  }

  fn immut_provide(&self, part: &[Inst], side: Side, mut starting_level: Level) -> LevelIterator {
    match part {
      [hd, tail @ ..] => {
        todo!()
      }
      [] => {
        let mut sub_iterators = vec![];
        for iface in self.side(side) {
          sub_iterators.push(iface.immut_provide(&part[1..], side, starting_level));
          starting_level += Level(iface.n_levels());
        }
        Box::new(ChainClone::new(sub_iterators))
      }
    }
  }

  fn realize<'db>(&self, _inst_time_args: Vec<&'db dyn std::any::Any>) -> Box<dyn Rtor<'db> + 'db> {
    let mut children = self
      .sctor
      .insts(self.db)
      .iter()
      .map(|inst| {
        (
          *inst,
          iface_of(self.db, inst.ctor(self.db)).comptime_realize(),
        )
      })
      .collect();
    connect(self.db, &mut children, self.sctor);
    fixpoint(&mut children);
    todo!()
  }

  fn n_levels(&self) -> u32 {
    todo!()
  }

  fn comptime_realize(&self) -> Box<dyn RtorComptime> {
    todo!()
  }
}
