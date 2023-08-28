use std::rc::Rc;

use crate::{ConnectionIterator, ProvidingConnectionIterator};

use super::{
  emptyiterator::EmptyIterator,
  nesting::{NBound, Nesting},
};

pub trait IteratorGiver<'a, Item, N: NBound> =
  Fn(Nesting<N>) -> Box<dyn ProvidingConnectionIterator<'a, Item = Item, N = N> + 'a>;

pub struct ChainClone<'a, Item, N: NBound, IG: IteratorGiver<'a, Item, N> + ?Sized> {
  backing_iters: Vec<Rc<IG>>,
  current: Box<dyn ProvidingConnectionIterator<'a, Item = Item, N = N> + 'a>,
  pos: u32,
}

impl<'a, Item: 'static, N: NBound + 'a, IG: IteratorGiver<'a, Item, N> + ?Sized>
  ChainClone<'a, Item, N, IG>
{
  pub fn new(mut nesting: Nesting<N>, iface: N, backing_iters: Vec<Rc<IG>>) -> Self {
    nesting.start_producer(iface);
    ChainClone {
      backing_iters,
      current: EmptyIterator::new_dyn(nesting),
      pos: 0,
    }
  }
}

impl<'a, Item, N: NBound, IG: IteratorGiver<'a, Item, N> + ?Sized> Iterator
  for ChainClone<'a, Item, N, IG>
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

impl<'a, Item, N: NBound, IG: IteratorGiver<'a, Item, N> + ?Sized> Clone
  for ChainClone<'a, Item, N, IG>
{
  fn clone(&self) -> Self {
    ChainClone {
      backing_iters: self.backing_iters.clone(),
      current: self.current.clone(),
      pos: self.pos,
    }
  }
}

impl<'a, Item, N: NBound, BackingIterator: IteratorGiver<'a, Item, N> + ?Sized>
  ConnectionIterator<'a> for ChainClone<'a, Item, N, BackingIterator>
{
  type N = N;
  fn current_nesting(&self) -> &Nesting<N> {
    todo!()
  }
}

impl<'a, Item, N: NBound, BackingIterator: IteratorGiver<'a, Item, N> + ?Sized>
  ProvidingConnectionIterator<'a> for ChainClone<'a, Item, N, BackingIterator>
{
  fn finish(self: Box<Self>) -> Nesting<N> {
    todo!()
  }
}
