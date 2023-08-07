pub struct LazyIterClone<'a, Item, CloneIterator: Iterator<Item = Item>>
where
  Box<CloneIterator>: Clone,
{
  backing_iter: &'a Option<Box<CloneIterator>>,
  backing_iter_clone: Option<Box<CloneIterator>>,
}
impl<'a, Item, CloneIterator: Iterator<Item = Item>> LazyIterClone<'a, Item, CloneIterator>
where
  Box<CloneIterator>: Clone,
{
  fn new(source: &'a Option<Box<CloneIterator>>) -> Self {
    LazyIterClone {
      backing_iter: source,
      backing_iter_clone: None,
    }
  }
}

impl<'a, Item, CloneIterator: Iterator<Item = Item>> Iterator
  for LazyIterClone<'a, Item, CloneIterator>
where
  Box<CloneIterator>: Clone,
{
  type Item = Item;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(iter) = self.backing_iter_clone.as_mut() {
      (*iter).next()
    } else {
      self.backing_iter_clone = Some(self.backing_iter.as_ref().unwrap().clone());
      self.next()
    }
  }
}
