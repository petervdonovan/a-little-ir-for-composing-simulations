use std::{
  cell::RefCell,
  collections::{hash_map::DefaultHasher, BTreeMap, HashMap, HashSet},
  hash::{Hash, Hasher},
  rc::Rc,
};

use crate::{
  iterators::{
    connectioniterator::ConnectionIterator,
    map::{map, pmap},
    nesting::Nesting,
  },
  rtor::{ComptimeInput, DeferredNotifys, FuzzySideIterator, Inputs, ProvidingInputsIface},
  Db,
};
use irlf_db::ir::{Inst, InstRef, StructlikeCtor};
use lf_types::{Comm, FlowDirection, Level, Side, SideMatch};

use crate::rtor::{InputsIface, LevelIterator, Rtor, RtorComptime, RtorIface};

use closure::closure;

use super::{iface_of, FixpointingStatus};
use crate::iterators::chainclone::ChainClone;

// dyn_clone::clone_trait_object!(ChainClone<Level, dyn LevelIterator<Item = Level>>);
// impl ConnectionIterator<Level> for ChainClone<Level, Box<dyn ConnectionIterator<Level>>> {}

pub struct Srtor<'db> {
  downstream: Option<Inputs<'db>>,
}

pub struct SrtorComptime<'a> {
  iface: SrtorIface,
  db: &'a dyn Db,
  levels_internal2external: Rc<RefCell<BTreeMap<Level, Level>>>,
  external_connections: Vec<(Vec<Inst>, Side, InputsIface<'a>)>,
}

#[salsa::tracked]
pub struct SrtorIface {
  sctor: StructlikeCtor,
}

struct SideIterator<'a> {
  ctor: StructlikeCtor,
  db: &'a dyn Db,
  side: SideMatch,
  pos: usize,
}

impl<'a> SideIterator<'a> {
  fn new(db: &'a dyn Db, ctor: StructlikeCtor, side: SideMatch) -> Self {
    SideIterator {
      ctor,
      db,
      side,
      pos: 0,
    }
  }
}

impl<'a> Iterator for SideIterator<'a> {
  type Item = Comm<InstRef>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let current = self.ctor.iface(self.db as &dyn irlf_db::Db).get(self.pos)?;
      self.pos += 1;
      if self.side.overlaps(current.0) {
        return Some(current.1.clone());
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

fn iface(sctor: StructlikeCtor, db: &dyn Db, side: SideMatch) -> SideIterator {
  SideIterator::new(db, sctor, side)
}

impl<'db> Rtor<'db> for Srtor<'db> {
  fn accept(&mut self, side: lf_types::Side, inputs: crate::rtor::Inputs<'db>) -> bool {
    todo!()
  }

  fn provide(&'db self, side: lf_types::Side, nesting: Nesting) -> crate::rtor::Inputs<'db> {
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
  children: &mut HashMap<Inst, Box<dyn RtorComptime<'db> + 'db>>,
  sctor: StructlikeCtor,
) {
  let mut connect =
    |ref_accept: Vec<Inst>, ref_provide: Vec<Inst>, side_accept: Side, side_provide: Side| {
      let mut provide1: InputsIface =
        children[&ref_provide[0]].provide(&ref_provide[1..], side_accept, Nesting::default());
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
  // todo: what if the children arrive at a level assignment that is incompatible with the ambient
  // TPO? You must add more constraints
  if changed == FixpointingStatus::Changed {
    fixpoint(children);
  }
}

/// An iterator over the ifaces of selected parts of a side.
struct StartingIntrinsicLevelProvider<'a, T, I: Iterator<Item = Comm<(Box<dyn RtorIface + 'a>, T)>>>
{
  it: I,
  db: &'a dyn Db,
  side: SideMatch,
  current_level: Level,
}

impl<'a, T, I: Iterator<Item = Comm<(Box<dyn RtorIface + 'a>, T)>>>
  StartingIntrinsicLevelProvider<'a, T, I>
{
  fn new(it: I, db: &'a dyn Db, side: SideMatch) -> Self {
    StartingIntrinsicLevelProvider {
      it,
      db,
      side,
      current_level: Level(0),
    }
  }
  fn n_levels(mut self) -> Level {
    for _ in self.by_ref() {}
    self.current_level
  }
}

impl<'a, T, I: Iterator<Item = Comm<(Box<dyn RtorIface + 'a>, T)>>> Iterator
  for StartingIntrinsicLevelProvider<'a, T, I>
{
  type Item = (Level, Box<dyn RtorIface + 'a>, T);

  fn next(&mut self) -> Option<Self::Item> {
    let ret = self.it.next()?;
    if let Comm::Data((ret, propagate)) = ret {
      let current_level = self.current_level;
      let delta = ret.n_levels(self.db, self.side);
      self.current_level += delta;
      Some((current_level, ret, propagate))
    } else {
      self.current_level += Level(1);
      self.next()
    }
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

fn rest_or_empty<T>(slice: &[T]) -> &[T] {
  if slice.is_empty() {
    slice
  } else {
    &slice[1..]
  }
}

impl<'a> RtorComptime<'a> for SrtorComptime<'a> {
  fn iterate_levels(&mut self) -> FixpointingStatus {
    // Do the accept. If fixpointing is just the same as doing the accept, then maybe we should not
    // bother with this function and instead just do the accept (connection) many times.
    let levels_map: Rc<RefCell<BTreeMap<Level, Level>>> = self.levels_internal2external.clone();
    let mut changed = FixpointingStatus::Unchanged;
    let mut deferred = DeferredNotifys::new();
    for (part, side, inputs) in self.external_connections.iter() {
      let mut cloned_inputs = inputs.clone();
      changed |= self.iface.immut_accept(
        self.db,
        part,
        *side,
        &mut map(
          &mut cloned_inputs,
          Rc::new(
            closure!(clone levels_map, |f: Comm<Rc<dyn Fn(Comm<Level>) -> FixpointingStatus>>| {
              f.map(|f: &Rc<dyn Fn(Comm<Level>) -> FixpointingStatus>| {
                let f = Rc::clone(f);
                // Note the extraction of `ret` into a local variable with a type annotation when it
                // could equivalently be returned immediately. This is either a bug or a weird
                // limitation of Rust's type inference system.
                let ret: Rc<dyn Fn(Comm<Level>) -> FixpointingStatus> = Rc::new(
                  closure!(clone levels_map, |intrinsic_lower_bound: Comm<Level>| {
                    (*f)(intrinsic_lower_bound.map(
                      |intrinsic_lower_bound| (*levels_map.borrow())[intrinsic_lower_bound]
                    ))
                  }));
                ret
              })
            }),
          ),
        ),
        &mut deferred,
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

  fn accept(&mut self, part: &[Inst], side: lf_types::Side, inputs: &mut InputsIface<'a>) {
    self
      .external_connections
      .push((part.to_vec(), side, inputs.clone()));
  }

  fn provide(
    &self,
    part: &[Inst],
    side: lf_types::Side,
    nesting: Nesting,
  ) -> ProvidingInputsIface<'a> {
    let mymap = Rc::clone(&self.levels_internal2external);
    let f = Rc::new(move |intrinsic_level: Comm<Level>| {
      let intrinsic_level = intrinsic_level.clone();
      let mymap = Rc::clone(&mymap);
      let f = move |lower_bound: Comm<Level>| match (lower_bound, intrinsic_level.clone()) {
        (Comm::Data(lower_bound), Comm::Data(intrinsic_level)) => {
          adjust(&mut mymap.borrow_mut(), lower_bound, intrinsic_level)
        }
        _ => FixpointingStatus::Unchanged,
      };
      ComptimeInput::Data(Rc::new(f)) as ComptimeInput
    });
    pmap(
      self
        .iface
        .immut_provide(self.db, part, side, Level(0), nesting),
      f,
    )
  }

  fn lower_bound(
    &mut self,
    part: &[Inst],
    side: Side,
    lower_bound: Level,
    last_direction: FlowDirection,
  ) {
    // TODO: is side_exact the right one?
    if let Some((level, _iface)) = self.iface.side_exact(self.db, side, part).next() {
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
    deferred_notifys: &mut DeferredNotifys,
  ) -> FixpointingStatus {
    let mut changed = FixpointingStatus::Unchanged;
    for (starting_intrinsic_level, iface) in self.side_exact(db, side, part) {
      if let Comm::Data(iface) = iface {
        changed |= iface.immut_accept(
          db,
          rest_or_empty(part),
          side,
          // &mut map(
          //   inputs_iface.clone(),
          //   Rc::new(move |it| {
          //     Rc::new(closure!(clone it, |level| it(level + starting_intrinsic_level)))
          //   }),
          // ),
          &mut map(
            inputs_iface, // FIXME: super wrong. will not work
            Rc::new(move |it| {
              it.map(|it| {
                // another case where the variable had to be factored out in order for it to type-check. Why?
                let it: Rc<dyn Fn(Comm<Level>) -> FixpointingStatus> = it.clone();
                let c: Rc<dyn Fn(Comm<Level>) -> FixpointingStatus> =
                  Rc::new(move |level: Comm<Level>| {
                    let ret: FixpointingStatus =
                      it(level.map(|level| *level + starting_intrinsic_level));
                    ret
                  });
                c
              })
            }),
          ),
          deferred_notifys,
        );

        // TODO: consume all level increments which are now at the head of the sequence and use them
        // to consume all the deferred notifys
      }
    }
    changed
  }

  fn immut_provide<'db>(
    &self,
    db: &'db dyn Db,
    part: &[Inst],
    side: Side,
    starting_level: Level,
    nesting: Nesting,
  ) -> LevelIterator<'db> {
    let mut sub_iterators: Vec<Rc<dyn Fn(Nesting) -> LevelIterator<'db> + 'db>> = vec![];
    let rc_part = Rc::new(part.to_vec());
    for (starting_intrinsic_level, iface) in self.side_exact(db, side, part) {
      match iface {
        Comm::Notify => todo!(),
        Comm::Data(iface) => {
          let rc_part = rc_part.clone();
          sub_iterators.push(Rc::new(move |nesting| {
            iface.immut_provide(
              db,
              rest_or_empty(&rc_part),
              side,
              starting_level + starting_intrinsic_level,
              nesting, // FIXME: How tf does this compile
            )
          }));
        }
      }
    }
    Box::new(ChainClone::new(nesting, Box::new(*self), sub_iterators))
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

  fn comptime_realize<'db>(&self, db: &'db dyn Db) -> Box<dyn RtorComptime<'db> + 'db> {
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
    for (starting_intrinsic_level, iface) in self.side_exact(db, side, part) {
      match iface {
        Comm::Notify => {
          ret.insert(starting_intrinsic_level);
        }
        Comm::Data(iface) => {
          ret.extend(iface.immut_provide_unique(
            db,
            rest_or_empty(part),
            side,
            starting_level + starting_intrinsic_level,
          ));
        }
      }
    }
    ret
  }
  fn side<'db>(
    &self,
    db: &'db dyn Db,
    side: SideMatch,
    part: &[Inst],
  ) -> Box<dyn Iterator<Item = (Level, SideMatch, Comm<Box<dyn RtorIface + 'db>>)> + 'db> {
    let part = part.to_vec();
    let ifaces = StartingIntrinsicLevelProvider::new(
      iface(self.sctor(db), db, side).map(|child| {
        child.map(|child| {
          let [child, tail @ ..] = &child.iref(db)[..] else {
            unreachable!("refs should never be empty if the ast passed parsing/validation")
          };
          let child = iface_of(db, child.ctor(db));
          (child, tail.to_vec())
        })
      }),
      db,
      side,
    )
    .filter_map(closure!(clone side, clone part, |(level, child, tail)| {
      let longer = sequence_max(&tail, rest_or_empty(&part))?;
      let ret: FuzzySideIterator<'db> = child.side(db, side, longer);
      Some(ret.map(move |(level_unadjusted, sm, iface)| (level_unadjusted + level, sm, iface)))
    }))
    .flatten();
    Box::new(ifaces)
  }

  fn n_levels(&self, db: &dyn Db, side: SideMatch) -> Level {
    // FIXME: there is some duplication here with side
    StartingIntrinsicLevelProvider::new(
      iface(self.sctor(db), db, side)
        .map(|it| it.map(|it| (iface_of(db, it.iref(db)[0].ctor(db)), ()))),
      db,
      side,
    )
    .n_levels()
  }

  fn iface_id(&self) -> u128 {
    let mut hasher = DefaultHasher::new();
    self.0.hash(&mut hasher);
    (hasher.finish() as u128) ^ 0xD245FC1824A8F73E483FDC370C0F7BD6
  }
}

#[salsa::tracked]
#[allow(clippy::borrowed_box)]
pub fn srtor_of(db: &dyn crate::Db, sctor: StructlikeCtor) -> Box<dyn RtorIface> {
  Box::new(SrtorIface::new(db, sctor))
}
