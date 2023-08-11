use std::{
  cell::RefCell,
  collections::{BTreeMap, HashMap, HashSet},
  hash::Hash,
  rc::Rc,
};

use irlf_db::{
  ir::{Connection, Inst, InstRef, StructlikeCtor},
  Db,
};
use lf_types::{Level, Side};

use crate::{
  iterators::cloneiterator::{map, CloneIterator},
  rtor::{
    InputsGiver, InputsIface, LevelIterator, Rtor, RtorComptime, RtorIface, ShareLevelLowerBound,
  },
};

use closure::closure;

use super::iface_of;
use crate::iterators::chainclone::ChainClone;

// dyn_clone::clone_trait_object!(ChainClone<Level, dyn LevelIterator<Item = Level>>);
// impl CloneIterator<Level> for ChainClone<Level, Box<dyn CloneIterator<Level>>> {}

pub struct Srtor<'db> {
  downstream: Option<InputsGiver<'db>>,
}

pub struct SrtorComptime<'a> {
  iface: SrtorIface<'a>,
  ldownstream: Option<InputsIface>,
  rdownstream: Option<InputsIface>,
  levels_internal2external: Rc<RefCell<BTreeMap<Level, Level>>>,
  sctor: StructlikeCtor,
  external_connections: Vec<(Vec<Inst>, Side, InputsIface)>,
}

#[derive(Clone)]
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
  type Item = InstRef;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let current = self.ctor.iface(self.db).get(self.pos)?;
      if self.side == current.0 {
        return Some(current.1);
      }
    }
  }
}

impl<'a> SrtorComptime<'a> {
  fn new(sctor: StructlikeCtor, iface: SrtorIface<'a>) -> Self {
    SrtorComptime {
      iface,
      ldownstream: None,
      rdownstream: None,
      levels_internal2external: Rc::new(RefCell::new(BTreeMap::new())),
      sctor,
      external_connections: vec![],
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
  fn side(&'a self, side: Side) -> Box<dyn Iterator<Item = Box<dyn RtorIface + 'a>> + 'a> {
    // Box::new(iface(self.sctor, self.db, side).map(|child| iface_of(self.db, child.ctor(self.db))))
    Box::new(iface(self.sctor, self.db, side).flat_map(|child| {
      child
        .iref(self.db)
        .iter()
        .map(|child| iface_of(self.db, child.ctor(self.db)))
        .collect::<Vec<_>>()
    }))
  }
}

fn adjust(
  map: &mut BTreeMap<Level, Level>,
  mut lower_bound: Level,
  intrinsic_level: Level,
) -> bool {
  let mut changed = false;
  for (_, l) in map.range_mut(intrinsic_level..) {
    if *l < lower_bound {
      lower_bound = *l + Level(1);
      *l = lower_bound;
      changed = true;
    } else {
      break;
    }
  }
  changed
}

impl<'a> RtorComptime for SrtorComptime<'a> {
  fn iterate_levels(&mut self) -> bool {
    // Do the accept. If fixpointing is just the same as doing the accept, then maybe we should not
    // bother with this function and instead just do the accept (connection) many times.
    let levels_map = self.levels_internal2external.clone();
    let mut changed = false;
    for (part, side, inputs) in self.external_connections.iter() {
      changed |= self.iface.immut_accept(
        part,
        *side,
        &mut map(
          inputs.clone(),
          Rc::new(closure!(clone levels_map, |f: Rc<dyn Fn(Level) -> bool>| {
            Rc::new(closure!(clone levels_map, |intrinsic_lower_bound| {
              (*f)((*levels_map.borrow())[&intrinsic_lower_bound])
            }))
          })),
        ),
      );
    }
    changed
  }

  fn levels(&self) -> HashSet<Level> {
    self
      .iface
      .levels()
      .into_iter()
      .map(|l| (*self.levels_internal2external.borrow())[&l])
      .collect()
  }

  fn accept(&mut self, part: &[Inst], side: lf_types::Side, inputs: &mut InputsIface) {
    self
      .external_connections
      .push((part.to_vec(), side, inputs.clone()));
  }

  fn provide(&self, part: &[Inst], side: lf_types::Side) -> InputsIface {
    let mymap = Rc::clone(&self.levels_internal2external);
    let f = Rc::new(move |intrinsic_level: Level| {
      let mymap = Rc::clone(&mymap);
      let f =
        move |lower_bound: Level| adjust(&mut mymap.borrow_mut(), lower_bound, intrinsic_level);
      Rc::new(f) as Rc<dyn Fn(Level) -> bool>
    });
    map(self.iface.immut_provide(part, side, Level(0)), f)
  }
}

impl<'a> RtorIface<'a> for SrtorIface<'a> {
  fn immut_accept(&self, part: &[Inst], side: Side, inputs_iface: &mut InputsIface) -> bool {
    let mut changed = false;
    for iface in self.side(side) {
      match part {
        [hd, tail @ ..] => {
          todo!()
        }
        [] => {
          for iface in self.side(side) {
            changed |= iface.immut_accept(part, side, inputs_iface);
          }
        }
      }
    }
    changed
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
    todo!("this can be implemented later after the level assignment algorithm passes unit tests")
  }

  fn n_levels(&self) -> u32 {
    // iterate over the left side and ask how many left levels everything needs, and over the right
    // side to ask how many right levels everything needs.
    todo!()
  }

  fn comptime_realize(&self) -> Box<dyn RtorComptime + 'a> {
    Box::new(SrtorComptime::new(self.sctor, self.clone()))
  }

  fn immut_provide_unique(
    &self,
    part: &[Inst],
    side: Side,
    mut starting_level: Level,
  ) -> HashSet<Level> {
    match part {
      [hd, tail @ ..] => {
        todo!()
      }
      [] => {
        let mut ret = HashSet::new();
        for iface in self.side(side) {
          ret.extend(iface.immut_provide_unique(&part[1..], side, starting_level));
          starting_level += Level(iface.n_levels());
        }
        ret
      }
    }
  }
}
