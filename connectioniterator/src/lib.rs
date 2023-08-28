#![feature(trait_alias)]

pub mod chainclone;
// pub mod connectioniterator;
pub mod emptyiterator;
pub mod lazyclone;
pub mod map;
pub mod nesting;

use dyn_clone::DynClone;

use crate::nesting::{NBound, Nesting};

pub trait ProvidingConnectionIterator<'a>: ConnectionIterator<'a> {
  /// Marks the termination of the iteration over `self` and returns the resulting Nesting.
  fn finish(self: Box<Self>) -> Nesting<Self::N>;
}

pub trait ConnectionIterator<'a>: Iterator + DynClone {
  type N: NBound;
  /// Returns the nesting corresponding to the latest output of self, or the initial nesting
  /// otherwise.
  fn current_nesting(&self) -> &Nesting<Self::N>;
}

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

#[cfg(test)]
mod tests {
  use std::rc::Rc;

  use crate::{chainclone::ChainClone, iterator_new, nesting::Nesting};
  use expect_test::expect;

  #[test]
  fn chain() {
    let closure_giver = |i, v: Vec<String>| move |n| iterator_new(n, i, v.clone());
    let stringify = |it: Vec<&str>| it.iter().map(|it| it.to_string()).collect();
    let chain = ChainClone::new(
      Nesting::default(),
      99,
      vec![
        Rc::new(closure_giver(43, stringify(vec!["a", "b", "c"]))),
        Rc::new(closure_giver(44, stringify(vec!["d", "e", "f"]))),
        Rc::new(closure_giver(45, stringify(vec!["h", "i", "j"]))),
        Rc::new(closure_giver(46, stringify(vec!["k", "l", "m"]))),
      ],
    );
    let s = format!("{:?}", chain.collect::<Vec<_>>());
    let expect = expect![[r#""#]];
    expect.assert_eq(&s);
  }
}
