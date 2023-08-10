use std::rc::Rc;

use dyn_clone::DynClone;

pub trait CloneIterator<Item>: Iterator<Item = Item> + DynClone {}

impl<T, Item> CloneIterator<Item> for T where T: Iterator<Item = Item> + DynClone {}

pub struct Map<T, Item>
where
  T: Iterator<Item = Item> + DynClone,
{
  backing_iterator: Box<T>,
  f: Rc<dyn Fn(Item) -> Item>,
}

impl<T, Item> Iterator for Map<T, Item>
where
  T: Iterator<Item = Item> + DynClone,
{
  type Item = Item;

  fn next(&mut self) -> Option<Self::Item> {
    Some((*self.f)(self.backing_iterator.next()?))
  }
}

impl<T, Item: 'static> Clone for Map<T, Item>
where
  T: Iterator<Item = Item> + DynClone,
{
  fn clone(&self) -> Self {
    Self {
      backing_iterator: dyn_clone::clone_box(&*self.backing_iterator),
      f: Rc::clone(&self.f),
    }
  }
}

pub fn map<'a, T, Item: 'static>(
  it: Box<T>,
  f: Rc<dyn Fn(Item) -> Item>,
) -> Box<dyn CloneIterator<Item> + 'a>
where
  T: Iterator<Item = Item> + DynClone + 'a,
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
