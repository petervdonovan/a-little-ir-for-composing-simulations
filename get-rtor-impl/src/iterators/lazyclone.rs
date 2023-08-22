use std::{cell::RefCell, rc::Rc};

pub struct LazyIterClone<Item, ConnectionIterator: Iterator<Item = Item> + ?Sized>
where
  Box<ConnectionIterator>: Clone,
{
  backing_iter: Rc<RefCell<Option<Box<ConnectionIterator>>>>,
  backing_iter_clone: Option<Box<ConnectionIterator>>,
}
impl<Item, ConnectionIterator: Iterator<Item = Item> + ?Sized>
  LazyIterClone<Item, ConnectionIterator>
where
  Box<ConnectionIterator>: Clone,
{
  pub fn new(source: Rc<RefCell<Option<Box<ConnectionIterator>>>>) -> Self {
    LazyIterClone {
      backing_iter: source,
      backing_iter_clone: None,
    }
  }
}

impl<Item, ConnectionIterator: Iterator<Item = Item> + ?Sized> Clone
  for LazyIterClone<Item, ConnectionIterator>
where
  Box<ConnectionIterator>: Clone,
{
  fn clone(&self) -> Self {
    LazyIterClone {
      backing_iter: self.backing_iter.clone(),
      backing_iter_clone: None,
    }
  }
}

impl<Item, ConnectionIterator: Iterator<Item = Item> + ?Sized> Iterator
  for LazyIterClone<Item, ConnectionIterator>
where
  Box<ConnectionIterator>: Clone,
{
  type Item = Item;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(iter) = self.backing_iter_clone.as_mut() {
      (*iter).next()
    } else {
      self.backing_iter_clone = Some(
        self
          .backing_iter
          .as_ref()
          .borrow()
          .as_ref()
          .unwrap()
          .clone(),
      );
      self.next()
    }
  }
}
