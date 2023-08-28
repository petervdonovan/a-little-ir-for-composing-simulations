use std::rc::Rc;

use crate::rtor::{EmptyIterator, RtorIface};

use super::{
  connectioniterator::{ConnectionIterator, ProvidingConnectionIterator},
  nesting::{NBound, Nesting},
};

pub struct ChainClone<
  'a,
  Item,
  N: NBound,
  IteratorGiver: Fn(Nesting<N>) -> Box<dyn ProvidingConnectionIterator<'a, Item = Item, N = N> + 'a> + ?Sized,
> {
  backing_iters: Vec<Rc<IteratorGiver>>,
  current: Box<dyn ProvidingConnectionIterator<'a, Item = Item, N = N> + 'a>,
  pos: u32,
}

impl<
    'a,
    Item: 'static,
    N: NBound + 'a,
    IteratorGiver: Fn(Nesting<N>) -> Box<dyn ProvidingConnectionIterator<'a, Item = Item, N = N> + 'a> + ?Sized,
  > ChainClone<'a, Item, N, IteratorGiver>
{
  pub fn new(mut nesting: Nesting<N>, iface: N, backing_iters: Vec<Rc<IteratorGiver>>) -> Self {
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
    N: NBound,
    IteratorGiver: Fn(Nesting<N>) -> Box<dyn ProvidingConnectionIterator<'a, Item = Item, N = N> + 'a> + ?Sized,
  > Iterator for ChainClone<'a, Item, N, IteratorGiver>
{
  type Item = Item;

  fn next(&mut self) -> Option<Self::Item> {
    // let current = self
    //   .current
    //   .get_or_insert_with(|| (*self.backing_iters.get_mut(self.pos as usize)?)());
    if let Some(x) = self.current.next() {
      Some(x)
    } else if let Some(next_giver) = self.backing_iters.get((self.pos + 1) as usize) {
      let moved = std::mem::replace(&mut self.current, next_giver(Nesting::default())); // FIXME: this is ugly
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
    N: NBound,
    IteratorGiver: Fn(Nesting<N>) -> Box<dyn ProvidingConnectionIterator<'a, Item = Item, N = N> + 'a> + ?Sized,
  > Clone for ChainClone<'a, Item, N, IteratorGiver>
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
    N: NBound,
    BackingIterator: Fn(Nesting<N>) -> Box<dyn ProvidingConnectionIterator<'a, Item = Item, N = N> + 'a> + ?Sized,
  > ConnectionIterator<'a> for ChainClone<'a, Item, N, BackingIterator>
{
  type N = N;
  fn current_nesting(&self) -> &Nesting<N> {
    todo!()
  }
}

impl<
    'a,
    Item,
    N: NBound,
    BackingIterator: Fn(Nesting<N>) -> Box<dyn ProvidingConnectionIterator<'a, Item = Item, N = N> + 'a> + ?Sized,
  > ProvidingConnectionIterator<'a> for ChainClone<'a, Item, N, BackingIterator>
{
  fn finish(self: Box<Self>) -> Nesting<N> {
    todo!()
  }
}
