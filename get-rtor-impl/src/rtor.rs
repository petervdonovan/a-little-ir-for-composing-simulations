use dyn_clone::DynClone;
use irlf_db::ir::Inst;
use lf_types::{FlowDirection, Level, Net, Side};
use std::{any::Any, collections::HashSet, marker::PhantomData, rc::Rc};

use crate::{iterators::cloneiterator::CloneIterator, rtorimpl::FixpointingStatus, Db};
// pub trait InputsIfaceIterator<'a>:
//   Iterator<Item = ShareLevelLowerBound<'a>> + 'a + DynClone
// {
// }
// dyn_clone::clone_trait_object!(InputsIfaceIterator<'_>);
// impl<'a> InputsIfaceIterator<'a> for dyn CloneIterator<ShareLevelLowerBound<'a>> {}
pub type SetPort<'db> = Box<dyn Fn(&dyn Any) + 'db>;
pub type Inputs<'a> = Box<dyn Iterator<Item = SetPort<'a>> + 'a>;
pub type InputsGiver<'a> = Box<dyn Fn() -> Inputs<'a> + 'a>;

pub type ShareLevelLowerBound = Rc<dyn Fn(Level) -> FixpointingStatus>;
dyn_clone::clone_trait_object!(CloneIterator<ShareLevelLowerBound>);
dyn_clone::clone_trait_object!(CloneIterator<Level>);
pub type InputsIface = Box<dyn CloneIterator<ShareLevelLowerBound>>;

pub type LevelIterator = Box<dyn CloneIterator<Level>>;
// pub trait LevelIterator: Iterator<Item = Level> + DynClone {}

pub struct EmptyIterator<'db, Item> {
  phantom: PhantomData<&'db Item>,
}
impl<'db, Item> Clone for EmptyIterator<'db, Item> {
  fn clone(&self) -> Self {
    EmptyIterator {
      phantom: PhantomData,
    }
  }
}
impl<'db, Item> Iterator for EmptyIterator<'db, Item> {
  type Item = Item;

  fn next(&mut self) -> Option<Self::Item> {
    None
  }
}
// impl<'a, Item> BoxClone for EmptyIterator<'a, Item> {}
// impl<'a> CloneIterator<ShareLevelLowerBound<'a>> for EmptyIterator<'a, ShareLevelLowerBound<'a>> {}
pub fn trivial_inputs_giver<'db>() -> Inputs<'db> {
  Box::new(EmptyIterator {
    phantom: PhantomData,
  })
}
pub fn trivial_inputs_iface_giver() -> InputsIface {
  let iterator = EmptyIterator::<ShareLevelLowerBound> {
    phantom: PhantomData,
  };
  Box::new(iterator)
}

/// A runtime reactor instance.
pub trait Rtor<'db> {
  /// Accepts the input of a downstream rtor. Returns true if the current rtor can now provide a
  /// different input. It must be safe to ignore the return value of this method.
  fn accept(&mut self, side: Side, inputs: InputsGiver<'db>) -> bool;
  /// Provides the inputs of this rtor.
  fn provide(&'db self, side: Side) -> InputsGiver<'db>;
  /// Steps this rtor forward by `distance` timesteps within the current nesting level.
  fn step_forward(&mut self, distance: u64) -> Option<Net>;
  /// Decrements the nesting level of this rtor's time.
  fn step_down(&mut self);
  /// Increments the nesting level of this rtor's time.
  fn step_up(&mut self) -> Option<Net>;
}

/// A potentially mutable compile-time model of a runtime `Rtor`.
///
/// `RtorComptime` instances **should not** instantiate `RtorComptime`s.
pub trait RtorComptime {
  /// Progresses the level of this and returns true if the value that would be produced by
  /// `self.levels` has changed. The correctness of this fixpointing feature is necessary for global
  /// correctness.
  fn iterate_levels(&mut self) -> FixpointingStatus;
  /// Requires that the levels of the given part of `self` be lower-bounded by `lower_bound`.
  /// Whether this bound is strict may depend on `last_direction`, the last flow direction on the
  /// given side.
  fn lower_bound(
    &mut self,
    part: &[Inst],
    side: Side,
    lower_bound: Level,
    last_direction: FlowDirection,
  );
  /// Returns the levels of the ambient program at which this reactor's local level is to be
  /// incremented.
  fn levels(&self) -> HashSet<Level>;
  /// Accepts the input of a downstream rtor. This is used for communication between the rtoriface
  /// instances about what the levels of their corresponding reactors should be.
  fn accept(&mut self, part: &[Inst], side: Side, inputs: &mut InputsIface);
  /// Provides the inputs of this rtor.
  fn provide(&self, part: &[Inst], side: Side) -> InputsIface;
}

/// An RtorIface can be instantiated by any entity that needs to know how a corresponding Rtor would
/// behave at runtime.
///
/// Implementors **should not** be mutable, i.e., they should not have cells nor should they be
/// designed for modification after initialization.
pub trait RtorIface: DynClone + std::fmt::Debug {
  /// The number of distinct levels required to model the given side of any instance of this rtor as
  /// a black box. This should be finite and trivial to compute.
  ///
  /// This will typically equal the initial flow direction, the number of internal flow direction
  /// changes, and the final flow direction, all on the given side only.
  fn n_levels(&self, db: &dyn Db, side: Side) -> (Level, Option<(FlowDirection, FlowDirection)>);
  /// States the levels at which an instance of self is to receive a TAGL.
  ///
  /// Similar to `immut_provide`, but finite and without repetition nor order nor a guarantee that
  /// it is exactly the same set of levels (although the numbers will be mostly the same).
  fn immut_provide_unique(
    &self,
    db: &dyn Db,
    part: &[Inst],
    side: Side,
    starting_level: Level,
  ) -> HashSet<Level>;
  /// Returns the levels at which an instance of `self` is to be notified of level advancement.
  fn levels(&self, db: &dyn Db) -> HashSet<Level> {
    let mut ret = HashSet::new();
    ret.extend(self.immut_provide_unique(db, &[], Side::Left, Level(0)));
    ret.extend(self.immut_provide_unique(db, &[], Side::Right, Level(0)));
    ret
  }
  /// If the intrinsic level $n$ of `self` sends to a given element $f$ of the provided
  /// `inputs_iface`, invoke $f$ on $n$. It is the responsibility of the caller to adjust the
  /// intrisic level $n$ according to any offsets as needed.
  ///
  /// `f` should be idempotent in the sense that invoking `f` multiple times on the same arguments
  /// should have the same effect as invoking `f` once on those arguments.
  ///
  /// Unlike `accept`, this function realizes effects on objects reachable from the closure of `f`
  /// rather than realizing effects on `self`.
  ///
  /// This function can be called by accept, but it should not call accept.
  ///
  /// Returns whether any of the callees in the `inputs_iface` said that a change resulted from the
  /// call.
  fn immut_accept(
    &self,
    db: &dyn Db,
    part: &[Inst],
    side: Side,
    inputs_iface: &mut InputsIface,
  ) -> FixpointingStatus;
  /// Returns the levels of the inputs of self.
  ///
  /// This function can be called by provide, but it should not call provide.
  fn immut_provide(
    &self,
    db: &dyn Db,
    part: &[Inst],
    side: Side,
    starting_level: Level,
  ) -> LevelIterator;
  /// Produces an instance of the RtorComptime associated with this.
  fn comptime_realize<'db>(&self, db: &'db dyn Db) -> Box<dyn RtorComptime + 'db>;
  /// Constructs an implementation given compile time and instantiation time args.
  fn realize<'db>(
    &self,
    db: &'db dyn Db,
    _inst_time_args: Vec<&'db dyn std::any::Any>,
  ) -> Box<dyn Rtor<'db> + 'db>;
  /// Returns the rtorifaces exposed on the given side by the given part of this.
  fn side<'db>(
    &self,
    db: &'db dyn Db,
    side: Side,
    part: &[Inst],
  ) -> Box<dyn Iterator<Item = (Level, Box<dyn RtorIface + 'db>)> + 'db>;
  fn iface_id(&self) -> u128;
}

dyn_clone::clone_trait_object!(RtorIface);

impl PartialEq for Box<dyn RtorIface> {
  fn eq(&self, other: &Self) -> bool {
    self.iface_id() == other.iface_id()
  }
}
impl Eq for Box<dyn RtorIface> {}
