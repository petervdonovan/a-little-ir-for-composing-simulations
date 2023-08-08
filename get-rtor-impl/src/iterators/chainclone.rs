pub struct ChainClone<Item, CloneIterator: Iterator<Item = Item> + ?Sized>
where
  Box<CloneIterator>: Clone,
{
  backing_iters: Vec<Box<CloneIterator>>,
  pos: u32,
}

impl<Item, CloneIterator: Iterator<Item = Item> + ?Sized> ChainClone<Item, CloneIterator>
where
  Box<CloneIterator>: Clone,
{
  pub fn new(backing_iters: Vec<Box<CloneIterator>>) -> Self {
    ChainClone {
      backing_iters,
      pos: 0,
    }
  }
}

impl<Item, CloneIterator: Iterator<Item = Item> + ?Sized> Iterator
  for ChainClone<Item, CloneIterator>
where
  Box<CloneIterator>: Clone,
{
  type Item = Item;

  fn next(&mut self) -> Option<Self::Item> {
    let current = self.backing_iters.get_mut(self.pos as usize)?;
    if let Some(x) = current.next() {
      Some(x)
    } else {
      self.pos += 1;
      self.next()
    }
  }
}

impl<Item, CloneIterator: Iterator<Item = Item> + ?Sized> Clone for ChainClone<Item, CloneIterator>
where
  Box<CloneIterator>: Clone,
{
  fn clone(&self) -> Self {
    Self {
      backing_iters: self.backing_iters.clone(),
      pos: self.pos,
    }
  }
}
