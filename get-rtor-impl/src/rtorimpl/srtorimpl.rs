use std::{
  cell::RefCell,
  collections::{BTreeMap, HashMap, HashSet},
  rc::Rc,
};

use crate::Db;
use irlf_db::ir::{Inst, InstRef, StructlikeCtor};
use lf_types::{FlowDirection, Level, Side};

use crate::{
  iterators::cloneiterator::map,
  rtor::{InputsGiver, InputsIface, LevelIterator, Rtor, RtorComptime, RtorIface},
};

use closure::closure;

use super::{iface_of, FixpointingStatus};
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
      // Conservatively start with a "ghost" entry at level zero because this instance _could_ have
      // inputs right before it in an interface with the same level as its starting level.
      levels_internal2external: Rc::new(RefCell::new(BTreeMap::from([(Level(0), Level(0))]))),
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
  let mut changed = FixpointingStatus::Unchanged;
  for (_, child) in children.iter_mut() {
    changed |= child.iterate_levels();
  }
  if changed == FixpointingStatus::Changed {
    fixpoint(children);
  }
  // todo: what if the children arrive at a level assignment that is incompatible with the ambient
  // TPO? You must add more constraints
}

/// An iterator over the ifaces of selected parts of a side.
struct StartingIntrinsicLevelProvider<'a, T, I: Iterator<Item = (Box<dyn RtorIface + 'a>, T)>> {
  it: I,
  db: &'a dyn Db,
  side: Side,
  current_level: Level,
  last_flow_directions: Option<(FlowDirection, FlowDirection)>,
}

impl<'a, T, I: Iterator<Item = (Box<dyn RtorIface + 'a>, T)>>
  StartingIntrinsicLevelProvider<'a, T, I>
{
  fn new(it: I, db: &'a dyn Db, side: Side) -> Self {
    StartingIntrinsicLevelProvider {
      it,
      db,
      side,
      current_level: Level(0),
      last_flow_directions: None,
    }
  }
  fn n_levels(mut self) -> (Level, Option<(FlowDirection, FlowDirection)>) {
    while self.last_flow_directions.is_none() {
      self.next();
    }
    let start_flow_direction = if let Some((s, _)) = self.last_flow_directions {
      Some(s)
    } else {
      None
    };
    for _ in self.by_ref() {}
    let flow_directions = if let Some(start) = start_flow_direction {
      // OK to unwrap here because if there is a start direction then there must be an end direction
      Some((start, self.last_flow_directions.unwrap().1))
    } else {
      None
    };
    (self.current_level, flow_directions)
  }
}

impl<'a, T, I: Iterator<Item = (Box<dyn RtorIface + 'a>, T)>> Iterator
  for StartingIntrinsicLevelProvider<'a, T, I>
{
  type Item = (Level, Box<dyn RtorIface + 'a>, T);

  fn next(&mut self) -> Option<Self::Item> {
    let (ret, propagate) = self.it.next()?;
    let current_level = self.current_level;
    let (delta, start_end) = ret.n_levels(self.db, self.side);
    self.current_level += delta;
    if let Some((start, _)) = start_end {
      self.current_level += Level(
        if let Some((_, d)) = self.last_flow_directions && d != start {
          // in order not to break abstraction we must conservatively assume that even when going
          // from input to output the level must be incremented because the output might depend on
          // multiple inputs from the last sequence of inputs. Of course, if the last sequence of
          // inputs consisted of only one atomic input, then we could do better, but currently even
          // that is abstracted.
          1
        } else {
          0
        });
      self.last_flow_directions = start_end;
    }
    Some((current_level, ret, propagate))
  }
}

fn sequence_max<'a>(a: &'a [Inst], b: &'a [Inst]) -> Option<&'a [Inst]> {
  let (short, long) = if b.len() > a.len() { (a, b) } else { (b, a) };
  if &long[0..short.len()] == short {
    Some(long)
  } else {
    None
  }
}

fn adjust(
  map: &mut BTreeMap<Level, Level>,
  mut lower_bound: Level,
  intrinsic_level: Level,
) -> FixpointingStatus {
  let mut changed = FixpointingStatus::Unchanged;
  for (_, l) in map.range_mut(intrinsic_level..) {
    if *l < lower_bound {
      lower_bound = *l + Level(1);
      *l = lower_bound;
      changed = FixpointingStatus::Changed;
    } else {
      break;
    }
  }
  changed
}

impl<'a> RtorComptime for SrtorComptime<'a> {
  fn iterate_levels(&mut self) -> FixpointingStatus {
    // Do the accept. If fixpointing is just the same as doing the accept, then maybe we should not
    // bother with this function and instead just do the accept (connection) many times.
    let levels_map: Rc<RefCell<BTreeMap<Level, Level>>> = self.levels_internal2external.clone();
    let mut changed = FixpointingStatus::Unchanged;
    for (part, side, inputs) in self.external_connections.iter() {
      changed |= self.iface.immut_accept(
        self.db,
        part,
        *side,
        &mut map(
          inputs.clone(),
          Rc::new(
            closure!(clone levels_map, |f: Rc<dyn Fn(Level) -> FixpointingStatus>| {
              Rc::new(closure!(clone levels_map, |intrinsic_lower_bound| {
                (*f)((*levels_map.borrow())[&intrinsic_lower_bound])
              }))
            }),
          ),
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
      Rc::new(f) as Rc<dyn Fn(Level) -> FixpointingStatus>
    });
    map(self.iface.immut_provide(self.db, part, side, Level(0)), f)
  }

  fn lower_bound(
    &mut self,
    part: &[Inst],
    side: Side,
    lower_bound: Level,
    last_direction: FlowDirection,
  ) {
    if let Some((level, _iface)) = self.iface.side(self.db, side, part).next() {
      adjust(
        &mut self.levels_internal2external.borrow_mut(),
        if last_direction == FlowDirection::In {
          lower_bound
        } else {
          lower_bound + Level(1)
        },
        level,
      );
    }
  }
}

impl RtorIface for SrtorIface {
  fn immut_accept(
    &self,
    db: &dyn Db,
    part: &[Inst],
    side: Side,
    inputs_iface: &mut InputsIface,
  ) -> FixpointingStatus {
    let mut changed = FixpointingStatus::Unchanged;
    for (starting_intrinsic_level, iface) in self.side(db, side, part) {
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

  fn immut_provide(
    &self,
    db: &dyn Db,
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

  fn comptime_realize<'db>(&self, db: &'db dyn Db) -> Box<dyn RtorComptime + 'db> {
    Box::new(SrtorComptime::new(*self, db))
  }

  fn immut_provide_unique(
    &self,
    db: &dyn Db,
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
    &self,
    db: &'db dyn Db,
    side: Side,
    part: &[Inst],
  ) -> Box<dyn Iterator<Item = (Level, Box<dyn RtorIface + 'db>)> + 'db> {
    let part = part.to_vec();
    let ifaces = StartingIntrinsicLevelProvider::new(iface(self.sctor(db), db, side)
      .map(|child| {
        let [child, tail @ ..] = &child.iref(db)[..] else { unreachable!("refs should never be empty if the ast passed parsing/validation") };
        let child = iface_of(db, child.ctor(db));
        (child, tail.to_vec())
      }), db, side)
      .filter_map(closure!(clone side, clone part, |(level, child, tail)| {
        let longer = sequence_max(&tail, &part[1..])?;
        // let iface = iface_of(db, longer[0].ctor(db));
        let ret: Box<dyn Iterator<Item = (Level, Box<dyn RtorIface + 'db>)>> = child.side(db, side, &longer[1..]);
        Some(ret.map(move |(level_unadjusted, iface)| (level_unadjusted + level, iface)))
      }))
      .flatten();
    Box::new(ifaces)
  }

  fn n_levels(&self, db: &dyn Db, side: Side) -> (Level, Option<(FlowDirection, FlowDirection)>) {
    // FIXME: there is some duplication here with side
    StartingIntrinsicLevelProvider::new(
      iface(self.sctor(db), db, side).map(|it| (iface_of(db, it.iref(db)[0].ctor(db)), ())),
      db,
      side,
    )
    .n_levels()
  }
}
