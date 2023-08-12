use std::{
  cell::RefCell,
  collections::{BTreeMap, HashMap, HashSet},
  rc::Rc,
};

use crate::Db;
use irlf_db::ir::{Inst, InstRef, StructlikeCtor};
use lf_types::{Level, Side};

use crate::{
  iterators::cloneiterator::map,
  rtor::{InputsGiver, InputsIface, LevelIterator, Rtor, RtorComptime, RtorIface},
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
  iface: SrtorIface,
  db: &'a dyn Db,
  levels_internal2external: Rc<RefCell<BTreeMap<Level, Level>>>,
  external_connections: Vec<(Vec<Inst>, Side, InputsIface)>,
}

#[salsa::tracked]
pub struct SrtorIface {
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
  fn new(iface: SrtorIface, db: &'a dyn Db) -> Self {
    SrtorComptime {
      iface,
      db,
      levels_internal2external: Rc::new(RefCell::new(BTreeMap::new())),
      external_connections: vec![],
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

/// An iterator over the ifaces of selected parts of a side.
struct SubSideIfaceIterator<'a, I: Iterator<Item = Box<dyn RtorIface>>> {
  it: I,
  db: &'a dyn Db,
  current_level: Level,
}

impl<'a, I: Iterator<Item = Box<dyn RtorIface>>> Iterator for SubSideIfaceIterator<'a, I> {
  type Item = (Level, Box<dyn RtorIface>);

  fn next(&mut self) -> Option<Self::Item> {
    let ret = self.it.next()?;
    let current_level = self.current_level;
    self.current_level += ret.n_levels(self.db);
    Some((current_level, ret))
  }
}

/// If `a` is a prefix of `b`, return the part of `b` not matched by `a`, and vice versa. This
/// function is symmetric wrt transposition of `a` and `b`.
fn prefix_match<'a>(a: &[Inst], b: &'a [Inst]) -> Option<&'a [Inst]> {
  match (a, b) {
    (_, []) => Some(&[]),
    ([], rest) => Some(rest),
    ([phd, ptl @ ..], [whd, wtl @ ..]) => {
      if phd == whd {
        prefix_match(ptl, wtl)
      } else {
        None
      }
    }
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
        self.db,
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

  fn levels<'db>(&self) -> HashSet<Level> {
    self
      .iface
      .levels(self.db)
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
    map(self.iface.immut_provide(self.db, part, side, Level(0)), f)
  }
}

impl RtorIface for SrtorIface {
  fn immut_accept<'db>(
    &self,
    db: &'db dyn Db,
    part: &[Inst],
    side: Side,
    inputs_iface: &mut InputsIface,
  ) -> bool {
    let mut changed = false;
    for (starting_intrinsic_level, iface) in self.side(db, side, part) {
      // TODO: map the inputs_iface and fold the conditional logic and the starting_intrinsic?_level into side
      changed |= iface.immut_accept(
        db,
        &part[1..],
        side,
        &mut map(
          inputs_iface.clone(),
          Rc::new(move |it| {
            Rc::new(closure!(clone it, |level| it(level + starting_intrinsic_level)))
          }),
        ),
      );
    }
    changed
  }

  fn immut_provide<'db>(
    &self,
    db: &'db dyn Db,
    part: &[Inst],
    side: Side,
    starting_level: Level,
  ) -> LevelIterator {
    let mut sub_iterators = vec![];
    for (starting_intrinsic_level, iface) in self.side(db, side, part) {
      sub_iterators.push(iface.immut_provide(
        db,
        &part[1..],
        side,
        starting_level + starting_intrinsic_level,
      ));
    }
    Box::new(ChainClone::new(sub_iterators))
  }

  fn realize<'db>(
    &self,
    db: &'db dyn Db,
    _inst_time_args: Vec<&'db dyn std::any::Any>,
  ) -> Box<dyn Rtor<'db> + 'db> {
    let mut children = self
      .sctor(db)
      .insts(db)
      .iter()
      .map(|inst| (*inst, iface_of(db, inst.ctor(db)).comptime_realize(db)))
      .collect();
    connect(db, &mut children, self.sctor(db));
    fixpoint(&mut children);
    todo!("this can be implemented later after the level assignment algorithm passes unit tests")
  }

  fn n_levels<'db>(&self, db: &'db dyn Db) -> Level {
    // iterate over the left side and ask how many left levels everything needs, and over the right
    // side to ask how many right levels everything needs.
    todo!()
  }

  fn comptime_realize<'db>(&self, db: &'db dyn Db) -> Box<dyn RtorComptime + 'db> {
    Box::new(SrtorComptime::new(*self, db))
  }

  fn immut_provide_unique<'db>(
    &self,
    db: &'db dyn Db,
    part: &[Inst],
    side: Side,
    starting_level: Level,
  ) -> HashSet<Level> {
    let mut ret = HashSet::new();
    for (starting_intrinsic_level, iface) in self.side(db, side, part) {
      ret.extend(iface.immut_provide_unique(
        db,
        &part[1..],
        side,
        starting_level + starting_intrinsic_level,
      ));
    }
    ret
  }
  fn side<'db>(
    &'db self,
    db: &'db dyn Db,
    side: Side,
    part: &[Inst],
  ) -> Box<dyn Iterator<Item = (Level, Box<dyn RtorIface>)>> {
    // let part = part.to_vec();
    // let it = iface(self.sctor, self.db, side)
    //   .map(move |child| child.iref(self.db))
    //   // child is nonempty because instrefs must be nonempty
    //   .filter(move |child| part.is_empty() || part[0] == child[0])
    //   .map(|child| iface_of(self.db, child[0].ctor(self.db)))
    //   // .collect::<Vec<_>>()
    //   ;
    // let ssifi = SubSideIfaceIterator {
    //   it,
    //   current_level: Level(0),
    // };
    // Box::new(ssifi)
    iface(self.sctor(db), db, side)
      .map(|child| child.iref(db))
      .filter_map(|child| {
        let tail = prefix_match(&child, part)?;
        let ret = iface_of(db, child[0].ctor(db)).side(db, side, tail);
        Some(ret)
      });
    todo!()
  }
}
