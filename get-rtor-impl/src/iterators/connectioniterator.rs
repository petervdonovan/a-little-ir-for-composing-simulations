use dyn_clone::DynClone;

use crate::rtor::RtorIface;

use super::nesting::Nesting;

pub trait ProvidingConnectionIterator<'a>: ConnectionIterator<'a> {
  // Marks the termination of the iteration over `self` and returns the resulting Nesting.
  fn finish(self: Box<Self>) -> Nesting;
}

pub trait ConnectionIterator<'a>: Iterator + DynClone {
  // Returns the nesting corresponding to the latest output of self, or the initial nesting
  // otherwise.
  fn current_nesting(&self) -> &Nesting;
}

// dyn_clone::clone_trait_object!(ConnectionIterator<Item = u32>);
impl<'a, Item> Clone for Box<dyn ConnectionIterator<'a, Item = Item> + 'a> {
  fn clone(&self) -> Self {
    dyn_clone::clone_box(&**self)
  }
}
impl<'a, Item> Clone for Box<dyn ProvidingConnectionIterator<'a, Item = Item> + 'a> {
  fn clone(&self) -> Self {
    dyn_clone::clone_box(&**self)
  }
}

// impl<T, Item> ConnectionIterator<Item> for T where T: Iterator<Item = Item> + DynClone {}

#[derive(Clone)]
struct VecIterator<Item> {
  v: Vec<Item>,
  pos: usize,
  nesting: Nesting,
}

impl<Item: Clone> Iterator for VecIterator<Item> {
  type Item = Item;

  fn next(&mut self) -> Option<Self::Item> {
    let ret = self.v.get(self.pos)?;
    self.pos += 1;
    Some(ret.clone())
  }
}
impl<'a, Item: Clone> ConnectionIterator<'a> for VecIterator<Item> {
  fn current_nesting(&self) -> &Nesting {
    &self.nesting
  }
}
impl<'a, Item: Clone> ProvidingConnectionIterator<'a> for VecIterator<Item> {
  fn finish(mut self: Box<Self>) -> Nesting {
    self.nesting.stop_producer();
    self.nesting
  }
}

pub fn iterator_new<'a, Item: Clone + 'static>(
  mut nesting: Nesting,
  iface: Box<dyn RtorIface>,
  v: Vec<Item>,
) -> Box<dyn ProvidingConnectionIterator<'a, Item = Item>> {
  nesting.start_producer(iface);
  Box::new(VecIterator { v, pos: 0, nesting })
}
