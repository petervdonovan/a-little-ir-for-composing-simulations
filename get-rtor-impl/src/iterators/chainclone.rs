use std::rc::Rc;

use crate::rtor::{EmptyIterator, RtorIface};

use super::{
  connectioniterator::{ConnectionIterator, ProvidingConnectionIterator},
  nesting::{Nesting, PLACEHOLDER},
};

pub struct ChainClone<
  'a,
  Item,
  IteratorGiver: Fn(Nesting) -> Box<dyn ProvidingConnectionIterator<'a, Item = Item> + 'a> + ?Sized,
> {
  backing_iters: Vec<Rc<IteratorGiver>>,
  current: Box<dyn ProvidingConnectionIterator<'a, Item = Item> + 'a>,
  pos: u32,
}

impl<
    'a,
    Item: 'static,
    IteratorGiver: Fn(Nesting) -> Box<dyn ProvidingConnectionIterator<'a, Item = Item> + 'a> + ?Sized,
  > ChainClone<'a, Item, IteratorGiver>
{
  pub fn new(
    mut nesting: Nesting,
    iface: Box<dyn RtorIface>,
    backing_iters: Vec<Rc<IteratorGiver>>,
  ) -> Self {
    nesting.start_producer(iface);
    ChainClone {
      backing_iters,
      current: EmptyIterator::new_dyn(nesting),
      pos: 0,
    }
  }
}

impl<
    'a,
    Item,
    IteratorGiver: Fn(Nesting) -> Box<dyn ProvidingConnectionIterator<'a, Item = Item> + 'a> + ?Sized,
  > Iterator for ChainClone<'a, Item, IteratorGiver>
{
  type Item = Item;

  fn next(&mut self) -> Option<Self::Item> {
    // let current = self
    //   .current
    //   .get_or_insert_with(|| (*self.backing_iters.get_mut(self.pos as usize)?)());
    if let Some(x) = self.current.next() {
      Some(x)
    } else if let Some(next_giver) = self.backing_iters.get((self.pos + 1) as usize) {
      let moved = std::mem::replace(&mut self.current, next_giver(PLACEHOLDER));
      self.current = next_giver(moved.finish());
      // let next = next_giver(self.current.finish());
      // self.current = next;
      self.pos += 1;
      self.next()
    } else {
      None
    }
  }
}

impl<
    'a,
    Item,
    IteratorGiver: Fn(Nesting) -> Box<dyn ProvidingConnectionIterator<'a, Item = Item> + 'a> + ?Sized,
  > Clone for ChainClone<'a, Item, IteratorGiver>
{
  fn clone(&self) -> Self {
    ChainClone {
      backing_iters: self.backing_iters.clone(),
      current: self.current.clone(),
      pos: self.pos,
    }
  }
}

impl<
    'a,
    Item,
    BackingIterator: Fn(Nesting) -> Box<dyn ProvidingConnectionIterator<'a, Item = Item> + 'a> + ?Sized,
  > ConnectionIterator<'a> for ChainClone<'a, Item, BackingIterator>
{
  fn current_nesting(&self) -> &Nesting {
    todo!()
  }
}

impl<
    'a,
    Item,
    BackingIterator: Fn(Nesting) -> Box<dyn ProvidingConnectionIterator<'a, Item = Item> + 'a> + ?Sized,
  > ProvidingConnectionIterator<'a> for ChainClone<'a, Item, BackingIterator>
{
  fn finish(self: Box<Self>) -> Nesting {
    todo!()
  }
}
