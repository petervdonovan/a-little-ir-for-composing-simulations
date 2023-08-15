use crate::iterators::cloneiterator::{iterator_new, map};
use crate::iterators::lazyclone::LazyIterClone;
use crate::rtor::{
  trivial_inputs_giver, trivial_inputs_iface_giver, InputsGiver, InputsIface, LevelIterator, Rtor,
  RtorComptime, RtorIface, SetPort,
};
use crate::Db;
use irlf_db::ir::{Inst, LibCtor};
use lf_types::{FlowDirection, Level, Net, Side, SideMatch};
use std::any::TypeId;
use std::cell::Cell;
use std::cmp;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::{any::Any, cell::RefCell, marker::PhantomData, rc::Rc};

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

struct FunRtorComptime {
  downstream: Rc<RefCell<Option<InputsIface>>>,
  level: Rc<Cell<Level>>,
}
struct FunRtor<'db> {
  downstream: Option<InputsGiver<'db>>,
  f: Rc<dyn Fn(u64) -> u64>,
  phantom: PhantomData<&'db u64>,
}

impl FunRtorIface {
  fn new<T: Fn(u64) -> u64 + 'static>(f: T) -> Self {
    let mut hasher = DefaultHasher::new();
    TypeId::of::<T>().hash(&mut hasher);
    let id = (hasher.finish() as u128) ^ 0x60F0D1407CF238F917600842BACE12A3;
    FunRtorIface { f: Rc::new(f), id }
  }
}

impl<'db> Rtor<'db> for FunRtor<'db> {
  fn accept(&mut self, side: Side, inputs: InputsGiver<'db>) -> bool {
    if let Side::Right = side {
      self.downstream = Some(inputs);
      false
    } else {
      false
    }
  }

  fn provide(&'db self, side: Side) -> InputsGiver<'db> {
    if let Side::Right = side {
      return Box::new(trivial_inputs_giver);
    }
    Box::new(|| {
      let fclone = self.f.clone();
      Box::new((self.downstream.as_ref().unwrap())().map(move |it| {
        let fcloneclone = fclone.clone();
        let mapped_it = move |sth: &dyn Any| {
          let sth = sth.downcast_ref::<u64>().unwrap();
          #[allow(clippy::redundant_closure_call)]
          let mapped = fcloneclone(*sth);
          (*it)(&mapped)
        };
        let b: SetPort<'db> = Box::new(mapped_it);
        b
      }))
    })
  }

  fn step_forward(&mut self, _distance: u64) -> Option<Net> {
    None
  }

  fn step_down(&mut self) {}

  fn step_up(&mut self) -> Option<Net> {
    None
  }
}

fn require_empty(part: &[Inst]) {
  if !part.is_empty() {
    panic!()
  }
}

impl RtorComptime for FunRtorComptime {
  fn iterate_levels(&mut self) -> FixpointingStatus {
    FixpointingStatus::Unchanged
  }
  fn levels(&self) -> HashSet<Level> {
    HashSet::new() // never notify; fn-like rtors react immediately
  }
  fn accept(&mut self, part: &[Inst], side: Side, inputs: &mut InputsIface) {
    require_empty(part);
    if let Side::Right = side {
      RefCell::replace(self.downstream.as_ref(), Some(inputs.clone()));
      inputs.next(); // ! This assumes that the width of self is 1 !
    }
  }
  fn provide(&self, part: &[Inst], side: Side) -> InputsIface {
    require_empty(part);
    // trivial_inputs_iface_giver()
    if let Side::Right = side {
      trivial_inputs_iface_giver()
    } else {
      let self_level = Rc::clone(&self.level);
      map(
        Box::new(LazyIterClone::new(Rc::clone(&self.downstream))),
        Rc::new(move |it| {
          let self_level = Rc::clone(&self_level);
          Rc::new(move |x| {
            self_level.replace(x);
            it(x)
          })
        }),
      )
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
    _part: &[Inst],
    _side: Side,
    _inputs_iface: &mut InputsIface,
  ) -> FixpointingStatus {
    FixpointingStatus::Unchanged // do nothing; self does not have any levels
  }
  fn immut_provide(
    &self,
    _db: &dyn Db,
    _part: &[Inst],
    _side: Side,
    starting_level: Level,
  ) -> LevelIterator {
    iterator_new(vec![starting_level])
  }

  fn n_levels(
    &self,
    _db: &dyn Db,
    side: SideMatch,
  ) -> (Level, Option<(FlowDirection, FlowDirection)>) {
    match side {
      SideMatch::One(Side::Left) => (Level(0), Some((FlowDirection::In, FlowDirection::In))),
      SideMatch::One(Side::Right) => (Level(0), Some((FlowDirection::Out, FlowDirection::Out))),
      SideMatch::Both => (Level(1), Some((FlowDirection::In, FlowDirection::Out))),
    }
  }

  fn comptime_realize(&self, _db: &dyn Db) -> Box<dyn RtorComptime> {
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
    _starting_level: Level,
  ) -> HashSet<Level> {
    HashSet::new() // never notify
  }

  fn side<'db>(
    &self,
    _db: &'db dyn Db,
    _side: SideMatch,
    _part: &[Inst],
  ) -> Box<dyn Iterator<Item = (Level, SideMatch, Box<dyn RtorIface + 'db>)> + 'db> {
    let cself: Box<dyn RtorIface> = Box::new(self.clone());
    Box::new(vec![(Level(0), SideMatch::Both, cself)].into_iter()) // FIXME: This assumes that the input width is only 1?
  }

  fn iface_id(&self) -> u128 {
    self.id
  }
}

#[salsa::tracked]
pub fn lctor_of(db: &dyn crate::Db, lctor: LibCtor) -> Box<dyn RtorIface> {
  match lctor.name(db).as_str() {
    "add1" => Box::new(FunRtorIface::new(|x| x + 1)),
    "mul2" => Box::new(FunRtorIface::new(|x| x * 2)),
    s => panic!("\"{s}\" is not an lctor name"),
  }
}
