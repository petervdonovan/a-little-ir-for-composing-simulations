use std::{
  any::TypeId,
  collections::{hash_map::DefaultHasher, HashSet},
  fmt::Debug,
  hash::{Hash, Hasher},
  rc::Rc,
};

use connectioniterator::{iterator_new, nesting::Nesting};
use irlf_db::ir::Inst;
use lf_types::{Comm, Level, Side, SideMatch};

use crate::rtor::{
  DeferredNotifys, InputsIface, LevelIterator, Rtor, RtorComptime, RtorIface, RtorN,
};

use super::{util::require_empty, FixpointingStatus};

#[derive(Clone)]
pub struct BiFunRtorIface {
  f: Rc<dyn Fn(u64, u64) -> u64>,
  id: u128,
}
impl BiFunRtorIface {
  pub fn new<T: Fn(u64, u64) -> u64 + 'static>(f: T) -> Self {
    let mut hasher = DefaultHasher::new();
    TypeId::of::<T>().hash(&mut hasher);
    let id = (hasher.finish() as u128) ^ 0xab28b6284f0552f30c158ea3265ea718;
    BiFunRtorIface { f: Rc::new(f), id }
  }
}

impl Debug for BiFunRtorIface {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("BiFunRtorIface")
      .field("f", &"???")
      .field("id", &self.id)
      .finish()
  }
}

impl RtorIface for BiFunRtorIface {
  fn n_levels(&self, _db: &dyn crate::Db, side: SideMatch) -> Level {
    match side {
      SideMatch::One(_) => Level(0),
      SideMatch::Both => Level(1),
    }
  }

  fn immut_provide_unique(
    &self,
    _db: &dyn crate::Db,
    part: &[Inst],
    side: Side,
    starting_level: Level,
  ) -> HashSet<Level> {
    require_empty(part);
    match side {
      Side::Left => HashSet::from([starting_level]),
      Side::Right => HashSet::from([starting_level + Level(1)]),
    }
  }

  fn immut_accept(
    &self,
    _db: &dyn crate::Db,
    part: &[Inst],
    side: Side,
    inputs_iface: &mut InputsIface,
    deferred_notifys: &mut DeferredNotifys,
  ) -> FixpointingStatus {
    require_empty(part);
    if side == Side::Right {
      let first = inputs_iface.next().unwrap();
      first.unwrap()(Comm::Data(Level(1)))
    } else {
      FixpointingStatus::Unchanged
    }
  }

  fn immut_provide<'db>(
    &self,
    _db: &'db dyn crate::Db,
    part: &[Inst],
    side: Side,
    starting_level: Level,
    nesting: Nesting<RtorN>,
  ) -> LevelIterator<'db> {
    require_empty(part);
    let ret = match side {
      Side::Left => vec![
        Comm::Data(starting_level),
        Comm::Data(starting_level),
        Comm::Notify,
      ],
      Side::Right => vec![Comm::Data(starting_level + Level(1))],
    };
    iterator_new(nesting, Box::new(self.clone()), ret)
  }

  fn comptime_realize<'db>(&self, db: &'db dyn crate::Db) -> Box<dyn RtorComptime<'db> + 'db> {
    todo!()
  }

  fn realize<'db>(
    &self,
    db: &'db dyn crate::Db,
    _inst_time_args: Vec<&'db dyn std::any::Any>,
  ) -> Box<dyn Rtor<'db> + 'db> {
    todo!()
  }

  fn side<'db>(
    &self,
    _db: &'db dyn crate::Db,
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
