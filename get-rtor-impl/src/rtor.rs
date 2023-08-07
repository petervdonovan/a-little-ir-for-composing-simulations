use dyn_clone::DynClone;
use lf_types::{Net, Side};
use std::{any::Any, marker::PhantomData};
pub trait InputsIfaceIterator<'a>:
  Iterator<Item = ShareLevelLowerBound<'a>> + 'a + DynClone
{
}
dyn_clone::clone_trait_object!(InputsIfaceIterator<'_>);
pub type SetPort<'db> = Box<dyn Fn(&dyn Any) + 'db>;
pub type ShareLevelLowerBound<'a> = Box<dyn Fn(u32) + 'a>;
pub type Inputs<'a> = Box<dyn Iterator<Item = SetPort<'a>> + 'a>;
pub type InputsGiver<'a> = Box<dyn Fn() -> Inputs<'a> + 'a>;
pub type InputsIface<'a> = Box<dyn InputsIfaceIterator<'a>>;

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
impl<'a> InputsIfaceIterator<'a> for EmptyIterator<'a, ShareLevelLowerBound<'a>> {}
pub fn trivial_inputs_giver<'db>() -> Inputs<'db> {
  Box::new(EmptyIterator {
    phantom: PhantomData,
  })
}
pub fn trivial_inputs_iface_giver<'a>() -> InputsIface<'a> {
  let iterator = EmptyIterator::<'a, ShareLevelLowerBound<'a>> {
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

/// An RtorIface can be instantiated by any entity that needs to know how a corresponding Rtor would
/// behave at runtime.
pub trait RtorIface<'a> {
  // fn new(ctor: Ctor, depth: u32, comp_time_args: Vec<&'db dyn Any>) -> Self;
  /// Accepts the input of a downstream rtor. This is used for communication between the rtoriface
  /// instances about what the levels of their corresponding reactors should be.
  fn accept(&'a mut self, side: Side, inputs: InputsIface<'a>);
  /// Provides the inputs of this rtor.
  fn provide(&'a self, side: Side) -> InputsIface<'a>;
  /// Progresses the level of this and returns true if the value that would be produced by
  /// `self.levels` has changed. The correctness of this fixpointing feature is necessary for global
  /// correctness.
  fn iterate_levels(&mut self) -> bool;
  /// Returns the levels of the ambient program at which this reactor's level is to be incremented.
  fn levels(&self) -> Vec<u32>;
  /// Constructs an implementation given compile time and instantiation time args.
  fn realize<'db>(&self, _inst_time_args: Vec<&'db dyn std::any::Any>) -> Box<dyn Rtor<'db> + 'db>;
}
