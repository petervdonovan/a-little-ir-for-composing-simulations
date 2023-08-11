use dyn_clone::DynClone;
use irlf_db::ir::Inst;
use lf_types::{Level, Net, Side};
use std::{any::Any, collections::HashSet, marker::PhantomData, rc::Rc};

use crate::iterators::cloneiterator::CloneIterator;
// pub trait InputsIfaceIterator<'a>:
//   Iterator<Item = ShareLevelLowerBound<'a>> + 'a + DynClone
// {
// }
// dyn_clone::clone_trait_object!(InputsIfaceIterator<'_>);
// impl<'a> InputsIfaceIterator<'a> for dyn CloneIterator<ShareLevelLowerBound<'a>> {}
pub type SetPort<'db> = Box<dyn Fn(&dyn Any) + 'db>;
pub type Inputs<'a> = Box<dyn Iterator<Item = SetPort<'a>> + 'a>;
pub type InputsGiver<'a> = Box<dyn Fn() -> Inputs<'a> + 'a>;

pub type ShareLevelLowerBound = Rc<dyn Fn(Level)>;
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
  fn iterate_levels(&mut self) -> bool;
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
pub trait RtorIface {
  /// The number of distinct levels required to model an instance of this rtor as a black box. This
  /// should be finite and trivial to compute.
  fn n_levels(&self) -> u32;
  /// States the levels at which an instance of self is to receive a TAGL.
  ///
  /// Similar to `immut_provide`, but finite and without repetition nor order nor a guarantee that
  /// it is exactly the same set of levels (although the numbers will be mostly the same).
  fn immut_provide_unique(
    &self,
    part: &[Inst],
    side: Side,
    starting_level: Level,
  ) -> HashSet<Level>;
  fn levels(&self) -> HashSet<Level> {
    let mut ret = HashSet::new();
    ret.extend(self.immut_provide_unique(&vec![], Side::Left, Level(0)));
    ret.extend(self.immut_provide_unique(&vec![], Side::Left, Level(0)));
    ret
  }
  /// If a level $n$ of an input provider receives from an output of self of level $k$, where $k$ is
  /// given with respect to the ambient level assignment of the immediately containing rtor of self,
  /// then invoke the impure function `f` on $(n, k)$. `f` should be idempotent in the sense that
  /// invoking `f` multiple times on the same arguments should have the same effect as invoking `f`
  /// once on those arguments.
  ///
  /// Unlike `accept`, this function realizes effects on objects reachable from the closure of `f`
  /// rather than realizing effects on `self`.
  ///
  /// This function can be called by accept, but it should not call accept.
  fn immut_accept(&self, part: &[Inst], side: Side, inputs_iface: &mut InputsIface);
  /// Returns the levels of the inputs of self.
  ///
  /// This function can be called by provide, but it should not call provide.
  fn immut_provide(&self, part: &[Inst], side: Side, starting_level: Level) -> LevelIterator;
  /// Produces an instance of the RtorComptime associated with this.
  fn comptime_realize(&self) -> Box<dyn RtorComptime>;
  /// Constructs an implementation given compile time and instantiation time args.
  fn realize<'db>(&self, _inst_time_args: Vec<&'db dyn std::any::Any>) -> Box<dyn Rtor<'db> + 'db>;
}
