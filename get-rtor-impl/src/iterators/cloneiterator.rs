use std::rc::Rc;

use dyn_clone::DynClone;

pub trait CloneIterator<Item>: Iterator<Item = Item> + DynClone {}

impl<T, Item> CloneIterator<Item> for T where T: Iterator<Item = Item> + DynClone {}

pub struct Map<T, ItemIn, ItemOut>
where
  T: Iterator<Item = ItemIn> + DynClone + ?Sized,
{
  backing_iterator: Box<T>,
  f: Rc<dyn Fn(ItemIn) -> ItemOut>,
}

impl<T, ItemIn, ItemOut> Iterator for Map<T, ItemIn, ItemOut>
where
  T: Iterator<Item = ItemIn> + DynClone + ?Sized,
{
  type Item = ItemOut;

  fn next(&mut self) -> Option<Self::Item> {
    Some((*self.f)(self.backing_iterator.next()?))
  }
}

impl<T, ItemIn: 'static, ItemOut: 'static> Clone for Map<T, ItemIn, ItemOut>
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

pub fn map<'a, T, ItemIn: 'static, ItemOut: 'static>(
  it: Box<T>,
  f: Rc<dyn Fn(ItemIn) -> ItemOut>,
) -> Box<dyn CloneIterator<ItemOut> + 'a>
where
  T: Iterator<Item = ItemIn> + DynClone + 'a + ?Sized,
  // Box<dyn Iterator<Item = Item>>: Clone,
{
  Box::new(Map {
    backing_iterator: it,
    f: Rc::clone(&f),
  })
}

#[derive(Clone)]
struct VecIterator<Item> {
  v: Vec<Item>,
  pos: usize,
}

impl<Item: Clone> Iterator for VecIterator<Item> {
  type Item = Item;

  fn next(&mut self) -> Option<Self::Item> {
    let ret = self.v.get(self.pos)?;
    self.pos += 1;
    Some(ret.clone())
  }
}

// impl<Item: Clone> CloneIterator<Item> for VecIterator<Item> {}

pub fn iterator_new<Item: Clone + 'static>(v: Vec<Item>) -> Box<dyn CloneIterator<Item>> {
  Box::new(VecIterator { v, pos: 0 })
}
