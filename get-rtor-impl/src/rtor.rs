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
pub type Inputs<'db> = Box<dyn Iterator<Item = SetPort<'db>> + 'db>;
pub type InputsGiver<'db> = Box<dyn Fn() -> Inputs<'db> + 'db>;

pub struct EmptyIterator<'db> {
  phantom: PhantomData<&'db u64>,
}
impl<'db> Iterator for EmptyIterator<'db> {
  type Item = SetPort<'db>;

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
  fn new(inst_time_args: Vec<&'db dyn Any>) -> Self;
  /// Accepts the input of a downstream rtor. Returns true if the current rtor can now provide a
  /// different input. It must be safe to ignore the return value of this method.
  fn accept(&mut self, side: Side, inputs: InputsGiver<'db>) -> bool;
  fn provide(&'db self, side: Side) -> InputsGiver<'db>;
  fn step_forward(&mut self, distance: u64) -> Option<Net>;
  fn step_down(&mut self);
  fn step_up(&mut self) -> Option<Net>;
}
