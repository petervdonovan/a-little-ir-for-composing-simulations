use std::{any::Any, marker::PhantomData};

pub enum Side {
  Left,
  Right,
}

pub enum Nesting {
  Up,
  Down,
}

pub type DeltaT = Vec<u64>;
pub type Net = DeltaT;
pub type SetPort<'db> = Box<dyn Fn(&dyn Any) + 'db>;
pub type ShareLevelLowerBound<'db> = Box<dyn Fn(u32) + 'db>;
pub type Inputs<'db> = Box<dyn Iterator<Item = (SetPort<'db>, ShareLevelLowerBound<'db>)> + 'db>;
pub type InputsGiver<'db> = Box<dyn Fn() -> Inputs<'db> + 'db>;

pub struct EmptyIterator<'db> {
  phantom: PhantomData<&'db u64>,
}
impl<'db> Iterator for EmptyIterator<'db> {
  type Item = (SetPort<'db>, ShareLevelLowerBound<'db>);

  fn next(&mut self) -> Option<Self::Item> {
    None
  }
}
pub fn trivial_inputs_giver<'db>() -> Inputs<'db> {
  Box::new(EmptyIterator {
    phantom: PhantomData,
  })
}

pub trait Rtor<'db> {
  /// Constructs an implementation given compile time and instantiation time args. This is a curried
  /// function.
  fn new(depth: u32, comp_time_args: Vec<&'db dyn Any>) -> Box<dyn Fn(Vec<&'db dyn Any>) -> Self>;
  /// Accepts the input of a downstream rtor. Returns true if the current rtor can now provide a
  /// different input. It must be safe to ignore the return value of this method.
  fn accept(&mut self, side: Side, inputs: InputsGiver<'db>) -> bool;
  /// Provides the inputs of this rtor.
  fn provide(&'db self, side: Side) -> InputsGiver<'db>;
  /// Steps this rtor forward by one timestep within the current nesting level.
  fn step_forward(&mut self, distance: u64) -> Option<Net>;
  /// Decrements the nesting level of this rtor's time.
  fn step_down(&mut self);
  /// Increments the nesting level of this rtor's time.
  fn step_up(&mut self) -> Option<Net>;
  /// Progresses the level of this and returns true if the value that would be produced by
  /// `self.levels` has changed. The correctness of this fixpointing feature is necessary for global
  /// correctness.
  fn iterate_levels(&mut self) -> bool;
  /// Returns the levels of the ambient program at which this reactor's level is to be incremented.
  fn levels(&self) -> Vec<u32>;
}
