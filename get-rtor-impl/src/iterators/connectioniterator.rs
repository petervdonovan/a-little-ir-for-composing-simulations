use dyn_clone::DynClone;

use crate::rtor::RtorIface;

use super::nesting::{NBound, Nesting};

pub trait ProvidingConnectionIterator<'a>: ConnectionIterator<'a> {
  // Marks the termination of the iteration over `self` and returns the resulting Nesting.
  fn finish(self: Box<Self>) -> Nesting<Self::N>;
}

pub trait ConnectionIterator<'a>: Iterator + DynClone {
  type N: NBound;
  // Returns the nesting corresponding to the latest output of self, or the initial nesting
  // otherwise.
  fn current_nesting(&self) -> &Nesting<Self::N>;
}

// dyn_clone::clone_trait_object!(ConnectionIterator<Item = u32>);
impl<'a, Item, N: NBound> Clone for Box<dyn ConnectionIterator<'a, Item = Item, N = N> + 'a> {
  fn clone(&self) -> Self {
    dyn_clone::clone_box(&**self)
  }
}
impl<'a, Item, N: NBound> Clone
  for Box<dyn ProvidingConnectionIterator<'a, Item = Item, N = N> + 'a>
{
  fn clone(&self) -> Self {
    dyn_clone::clone_box(&**self)
  }
}

// impl<T, Item> ConnectionIterator<Item> for T where T: Iterator<Item = Item> + DynClone {}

#[derive(Clone)]
struct VecIterator<Item, N: NBound> {
  v: Vec<Item>,
  pos: usize,
  nesting: Nesting<N>,
}

impl<Item: Clone, N: NBound> Iterator for VecIterator<Item, N> {
  type Item = Item;

  fn next(&mut self) -> Option<Self::Item> {
    let ret = self.v.get(self.pos)?;
    self.pos += 1;
    Some(ret.clone())
  }
}
impl<'a, Item: Clone, N: NBound> ConnectionIterator<'a> for VecIterator<Item, N> {
  type N = N;
  fn current_nesting(&self) -> &Nesting<N> {
    &self.nesting
  }
}
impl<'a, Item: Clone, N: NBound> ProvidingConnectionIterator<'a> for VecIterator<Item, N> {
  fn finish(mut self: Box<Self>) -> Nesting<N> {
    self.nesting.stop_producer();
    self.nesting
  }
}

pub fn iterator_new<'a, Item: Clone + 'static, N: NBound + 'static>(
  mut nesting: Nesting<N>,
  iface: N,
  v: Vec<Item>,
) -> Box<dyn ProvidingConnectionIterator<'a, Item = Item, N = N>> {
  nesting.start_producer(iface);
  Box::new(VecIterator { v, pos: 0, nesting })
}
