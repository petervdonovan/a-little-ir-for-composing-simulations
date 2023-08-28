use std::marker::PhantomData;

use crate::{ConnectionIterator, ProvidingConnectionIterator};

use super::nesting::{NBound, Nesting};

pub struct EmptyIterator<'a, Item, N: NBound> {
  nesting: Nesting<N>,
  phantom: PhantomData<&'a Item>,
}
impl<'a, Item: 'static, N: NBound + 'a> EmptyIterator<'a, Item, N> {
  pub fn new_dyn(
    nesting: Nesting<N>,
  ) -> Box<dyn ProvidingConnectionIterator<'a, Item = Item, N = N> + 'a> {
    Box::new(EmptyIterator {
      nesting,
      phantom: PhantomData,
    })
  }
}
impl<'a, Item, N: NBound> Clone for EmptyIterator<'a, Item, N> {
  fn clone(&self) -> Self {
    EmptyIterator {
      nesting: self.nesting.clone(),
      phantom: PhantomData,
    }
  }
}
impl<'a, Item, N: NBound> Iterator for EmptyIterator<'a, Item, N> {
  type Item = Item;

  fn next(&mut self) -> Option<Self::Item> {
    None
  }
}

impl<'a, Item, N: NBound> ConnectionIterator<'a> for EmptyIterator<'a, Item, N> {
  type N = N;
  fn current_nesting(&self) -> &Nesting<N> {
    todo!()
  }
}

impl<'a, Item, N: NBound> ProvidingConnectionIterator<'a> for EmptyIterator<'a, Item, N> {
  fn finish(self: Box<Self>) -> Nesting<N> {
    todo!()
  }
}
