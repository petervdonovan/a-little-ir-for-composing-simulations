use std::rc::Rc;

use dyn_clone::DynClone;

use super::{
  connectioniterator::{ConnectionIterator, ProvidingConnectionIterator},
  nesting::Nesting,
};

enum BackingIterator<'a, T: ?Sized> {
  Borrowed(&'a mut Box<T>),
  Owned(Box<T>),
}

struct Map<'a, T, ItemIn, ItemOut>
where
  T: Iterator<Item = ItemIn> + DynClone + ?Sized,
{
  backing_iterator: BackingIterator<'a, T>,
  f: Rc<dyn Fn(ItemIn) -> ItemOut>,
}

impl<'a, T, ItemIn, ItemOut> Map<'a, T, ItemIn, ItemOut>
where
  T: Iterator<Item = ItemIn> + DynClone + ?Sized,
{
  #[allow(clippy::borrowed_box)]
  fn backing_iterator(&self) -> &Box<T> {
    match &self.backing_iterator {
      BackingIterator::Borrowed(ref ret) => ret,
      BackingIterator::Owned(ref ret) => ret,
    }
  }
  fn backing_iterator_mut(&mut self) -> &mut Box<T> {
    match &mut self.backing_iterator {
      BackingIterator::Borrowed(ref mut ret) => ret,
      BackingIterator::Owned(ref mut ret) => ret,
    }
  }
}

struct ProvidingMap<T, ItemIn, ItemOut>
where
  T: Iterator<Item = ItemIn> + DynClone + ?Sized,
{
  backing_iterator: Box<T>,
  f: Rc<dyn Fn(ItemIn) -> ItemOut>,
}

impl<'a, 'b, T, ItemIn: 'static, ItemOut: 'static> ConnectionIterator<'a>
  for ProvidingMap<T, ItemIn, ItemOut>
where
  T: ConnectionIterator<'b, Item = ItemIn> + ?Sized,
{
  fn current_nesting(&self) -> &Nesting {
    self.backing_iterator.current_nesting()
  }
}

impl<'a, T, ItemIn: 'static, ItemOut: 'static> ProvidingConnectionIterator<'a>
  for ProvidingMap<T, ItemIn, ItemOut>
where
  T: ProvidingConnectionIterator<'a, Item = ItemIn> + ?Sized,
{
  fn finish(self: Box<Self>) -> Nesting {
    self.backing_iterator.finish()
  }
}

impl<'a, T, ItemIn, ItemOut> Iterator for Map<'a, T, ItemIn, ItemOut>
where
  T: Iterator<Item = ItemIn> + DynClone + ?Sized,
{
  type Item = ItemOut;

  fn next(&mut self) -> Option<Self::Item> {
    let next = self.backing_iterator_mut().next()?;
    Some((*self.f)(next))
  }
}

impl<T, ItemIn, ItemOut> Iterator for ProvidingMap<T, ItemIn, ItemOut>
where
  T: Iterator<Item = ItemIn> + DynClone + ?Sized,
{
  type Item = ItemOut;

  fn next(&mut self) -> Option<Self::Item> {
    let next = self.backing_iterator.next()?;
    Some((*self.f)(next))
  }
}

impl<'a, 'b, T, ItemIn: 'static, ItemOut: 'static> ConnectionIterator<'a>
  for Map<'a, T, ItemIn, ItemOut>
where
  T: ConnectionIterator<'b, Item = ItemIn> + ?Sized,
{
  fn current_nesting(&self) -> &Nesting {
    self.backing_iterator().current_nesting()
  }
}

impl<T, ItemIn: 'static, ItemOut: 'static> Clone for ProvidingMap<T, ItemIn, ItemOut>
where
  T: Iterator<Item = ItemIn> + DynClone + ?Sized,
{
  fn clone(&self) -> Self {
    Self {
      backing_iterator: dyn_clone::clone_box(&*self.backing_iterator),
      f: Rc::clone(&self.f),
    }
  }
}

impl<'a, T, ItemIn: 'static, ItemOut: 'static> Clone for Map<'a, T, ItemIn, ItemOut>
where
  T: Iterator<Item = ItemIn> + DynClone + ?Sized,
{
  fn clone(&self) -> Self {
    Self {
      backing_iterator: BackingIterator::Owned(dyn_clone::clone_box(self.backing_iterator())),
      f: Rc::clone(&self.f),
    }
  }
}

pub fn map<'a, 'b: 'a, T, ItemIn: 'static, ItemOut: 'static>(
  it: &'a mut Box<T>,
  f: Rc<dyn Fn(ItemIn) -> ItemOut>,
) -> Box<dyn ConnectionIterator<'a, Item = ItemOut> + 'a>
where
  T: ConnectionIterator<'b, Item = ItemIn> + 'b + ?Sized,
{
  let ret: Box<dyn ConnectionIterator<'a, Item = ItemOut> + 'a> = Box::new(Map {
    backing_iterator: BackingIterator::Borrowed(it),
    f: Rc::clone(&f),
  });
  ret
}

pub fn pmap<'a, T, ItemIn: 'static, ItemOut: 'static>(
  it: Box<T>,
  f: Rc<dyn Fn(ItemIn) -> ItemOut>,
) -> Box<dyn ProvidingConnectionIterator<'a, Item = ItemOut> + 'a>
where
  T: ProvidingConnectionIterator<'a, Item = ItemIn> + 'a + ?Sized,
{
  Box::new(ProvidingMap {
    backing_iterator: it,
    f: Rc::clone(&f),
  })
}
