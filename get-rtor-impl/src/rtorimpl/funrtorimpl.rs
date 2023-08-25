use crate::iterators::connectioniterator::{iterator_new, Nesting};
use crate::rtor::{
  DeferredNotifys, EmptyIterator, Inputs, InputsIface, LevelIterator, Rtor, RtorComptime,
  RtorIface, SetPort,
};
use crate::Db;
use irlf_db::ir::Inst;
use lf_types::{Comm, FlowDirection, Level, Net, Side, SideMatch};
use std::any::TypeId;
use std::cell::Cell;
use std::cmp;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use super::util::require_empty;
use super::FixpointingStatus;

#[derive(Clone)]
pub struct FunRtorIface {
  f: Rc<dyn Fn(u64) -> u64>,
  id: u128,
}

impl std::fmt::Debug for FunRtorIface {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("FunRtorIface").field("f", &"??").finish()
  }
}

struct FunRtorComptime<'a> {
  downstream: Rc<RefCell<Option<InputsIface<'a>>>>,
  level: Rc<Cell<Level>>,
}
struct FunRtor<'db> {
  downstream: Option<Inputs<'db>>,
  f: Rc<dyn Fn(u64) -> u64>,
  phantom: PhantomData<&'db u64>,
}

impl FunRtorIface {
  pub fn new<T: Fn(u64) -> u64 + 'static>(f: T) -> Self {
    let mut hasher = DefaultHasher::new();
    TypeId::of::<T>().hash(&mut hasher);
    let id = (hasher.finish() as u128) ^ 0x60F0D1407CF238F917600842BACE12A3;
    FunRtorIface { f: Rc::new(f), id }
  }
}

impl<'db> Rtor<'db> for FunRtor<'db> {
  fn accept(&mut self, side: Side, inputs: Inputs<'db>) -> bool {
    if let Side::Right = side {
      self.downstream = Some(inputs);
      false
    } else {
      false
    }
  }

  fn provide(&'db self, side: Side, nesting: Nesting) -> Inputs<'db> {
    // if let Side::Right = side {
    //   return EmptyIterator::new(nesting);
    // }
    // let fclone = self.f.clone();
    todo!()
    // map(*self.downstream.as_ref().unwrap(), move |it| {
    //   let fcloneclone = fclone.clone();
    //   let mapped_it = move |sth: &dyn Any| {
    //     let sth = sth.downcast_ref::<u64>().unwrap();
    //     #[allow(clippy::redundant_closure_call)]
    //     let mapped = fcloneclone(*sth);
    //     (*it)(&mapped)
    //   };
    //   let b: SetPort<'db> = Box::new(mapped_it);
    //   b
    // })
  }

  fn step_forward(&mut self, _distance: u64) -> Option<Net> {
    None
  }

  fn step_down(&mut self) {}

  fn step_up(&mut self) -> Option<Net> {
    None
  }
}

impl<'a> RtorComptime<'a> for FunRtorComptime<'a> {
  fn iterate_levels(&mut self) -> FixpointingStatus {
    FixpointingStatus::Unchanged
  }
  fn levels(&self) -> HashSet<Level> {
    HashSet::new() // never notify; fn-like rtors react immediately
  }
  fn accept(&mut self, part: &[Inst], side: Side, inputs: &mut InputsIface<'a>) {
    require_empty(part);
    if let Side::Right = side {
      RefCell::replace(self.downstream.as_ref(), Some(inputs.clone()));
      inputs.next(); // ! This assumes that the width of self is 1 !
    }
  }
  fn provide(&self, part: &[Inst], side: Side, nesting: Nesting) -> InputsIface<'a> {
    require_empty(part);
    // trivial_inputs_iface_giver()
    if let Side::Right = side {
      EmptyIterator::new_dyn(nesting)
    } else {
      // let self_level = Rc::clone(&self.level);
      // map(
      //   Box::new(LazyIterClone::new(Rc::clone(&self.downstream))),
      //   Rc::new(move |it: lf_types::Comm<Rc<dyn Fn(lf_types::Comm<Level>) -> FixpointingStatus>>| {
      //     let self_level = Rc::clone(&self_level);
      //     let c: dyn Fn(Level) -> IIEltE = move |x| {
      //       self_level.replace(x);
      //       it(x)
      //     };
      //     Rc::new(c)
      //   }),
      // )
      todo!()
    }
  }

  fn lower_bound(
    &mut self,
    part: &[Inst],
    side: Side,
    lower_bound: Level,
    last_direction: FlowDirection,
  ) {
    require_empty(part);
    if side == Side::Left {
      let nonstrict = cmp::max(lower_bound, self.level.get());
      let strict = nonstrict + Level(1);
      self.level.replace(if last_direction == FlowDirection::Out {
        nonstrict
      } else {
        strict
      });
    }
  }
}

impl RtorIface for FunRtorIface {
  fn immut_accept(
    &self,
    _db: &dyn Db,
    part: &[Inst],
    side: Side,
    inputs_iface: &mut InputsIface,
    deferred_notifys: &mut DeferredNotifys,
  ) -> FixpointingStatus {
    require_empty(part);
    if side == Side::Right {
      let first = inputs_iface.next().unwrap();
      // FIXME: The right thing to do here is probably to find the first element of the iface that
      // is not a Notify
      first.unwrap()(Comm::Data(Level(0)))
    } else {
      FixpointingStatus::Unchanged
    }
  }
  fn immut_provide<'db>(
    &self,
    _db: &'db dyn Db,
    _part: &[Inst],
    _side: Side,
    starting_level: Level,
    nesting: Nesting,
  ) -> LevelIterator<'db> {
    iterator_new(
      nesting,
      Box::new(self.clone()),
      vec![Comm::Data(starting_level)],
    )
  }

  fn n_levels(&self, _db: &dyn Db, _side: SideMatch) -> Level {
    Level(0)
  }

  fn comptime_realize<'db>(&self, _db: &'db dyn Db) -> Box<dyn RtorComptime<'db> + 'db> {
    Box::new(FunRtorComptime {
      downstream: Rc::new(RefCell::new(None)),
      level: Rc::new(Cell::new(Level(0))), // TODO: check?
    })
  }
  fn realize<'db>(
    &self,
    _db: &'db dyn Db,
    _inst_time_args: Vec<&'db dyn std::any::Any>,
  ) -> Box<dyn Rtor<'db> + 'db> {
    Box::new(FunRtor {
      downstream: None,
      phantom: PhantomData,
      f: Rc::clone(&self.f),
    })
  }

  fn immut_provide_unique(
    &self,
    _db: &dyn Db,
    _part: &[Inst],
    _side: Side,
    starting_level: Level,
  ) -> HashSet<Level> {
    HashSet::from([starting_level]) // never notify
  }

  fn side<'db>(
    &self,
    _db: &'db dyn Db,
    _side: SideMatch,
    part: &[Inst],
  ) -> Box<dyn Iterator<Item = (Level, SideMatch, Comm<Box<dyn RtorIface + 'db>>)> + 'db> {
    require_empty(part);
    let cself: Box<dyn RtorIface> = Box::new(self.clone());
    Box::new(vec![(Level(0), SideMatch::Both, Comm::Data(cself))].into_iter())
    // FIXME: This assumes that the input width is only 1?
  }

  fn iface_id(&self) -> u128 {
    self.id
  }
}
