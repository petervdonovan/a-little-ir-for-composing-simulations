use std::rc::Rc;

use dyn_clone::DynClone;

use crate::rtor::RtorIface;

#[derive(Debug, Clone)]
pub struct NestingStack(Vec<Box<dyn RtorIface>>);

#[derive(Debug, Clone, Copy)]
pub struct Cursor(usize);

#[derive(Debug, Clone)]
pub struct Nesting {
  all: NestingStack,
  // invariant: cursors[0] == 0
  cursors: Vec<Cursor>,
}

// TODO: This is used as a placeholder to allow memory to be temporarily in an invalid state and it
// seems very hacky and I do not think that it achieves the performance and static checking benefit
// that is desired. All uses of it should be examined for sus-ness.
pub const PLACEHOLDER: Nesting = Nesting {
  all: NestingStack(vec![]),
  cursors: vec![],
};

impl Default for Nesting {
  fn default() -> Self {
    Self {
      all: NestingStack(vec![]),
      cursors: vec![Cursor(0)],
    }
  }
}

impl NestingStack {
  pub fn active(&self, cursor: Cursor) -> Option<&[Box<dyn RtorIface>]> {
    if cursor.0 < self.0.len() {
      Some(&self.0[cursor.0..])
    } else {
      None
    }
  }
}

impl Nesting {
  pub fn active(&self) -> Option<&[Box<dyn RtorIface>]> {
    if let Some(cursor) = self.cursors.last() {
      self.all.active(*cursor)
    } else {
      panic!("there should always be at least one cursor");
    }
  }
  pub fn start_consumer(&mut self) {
    self.cursors.push(Cursor(self.all.0.len()));
  }
  pub fn stop_consumer(&mut self) {
    self.cursors.pop();
  }
  pub fn start_producer(&mut self, producer: Box<dyn RtorIface>) {
    self.all.0.push(producer);
  }
  pub fn stop_producer(&mut self) {
    self.all.0.pop();
  }
}

pub trait ConnectionIterator<'a>: Iterator + DynClone {
  // Returns the nesting corresponding to the latest output of self, or the initial nesting
  // otherwise.
  fn current_nesting(&self) -> &Nesting;
  // Marks the termination of the iteration over `self` and returns the resulting Nesting.
  fn finish(self: Box<Self>) -> Nesting;
}

// dyn_clone::clone_trait_object!(ConnectionIterator<Item = u32>);
impl<'a, Item> Clone for Box<dyn ConnectionIterator<'a, Item = Item> + 'a> {
  fn clone(&self) -> Self {
    dyn_clone::clone_box(&**self)
  }
}

// impl<T, Item> ConnectionIterator<Item> for T where T: Iterator<Item = Item> + DynClone {}
enum BackingIterator<'a, T: ?Sized> {
  Owned(Box<T>),
  Borrowed(&'a mut Box<T>),
}
pub struct Map<'a, T, ItemIn, ItemOut>
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
  fn backing_iterator_mut<Out>(&mut self, op: impl Fn(&mut Box<T>) -> Out) -> Out {
    match &mut self.backing_iterator {
      BackingIterator::Owned(ref mut mine) => op(mine),
      BackingIterator::Borrowed(theirs) => op(*theirs),
    }
  }
  fn backing_iterator<'b, Out>(&'b self, op: impl Fn(&'b Box<T>) -> Out) -> Out {
    match &self.backing_iterator {
      BackingIterator::Owned(mine) => op(mine),
      BackingIterator::Borrowed(theirs) => op(*theirs),
    }
  }
  fn backing_iterator_move<'b, Out>(self, op: impl Fn(Box<T>) -> Out) -> Out {
    match self.backing_iterator {
      BackingIterator::Owned(mine) => op(mine),
      BackingIterator::Borrowed(theirs) => op(*theirs),
    }
  }
}

impl<'a, T, ItemIn: 'static, ItemOut: 'static> ConnectionIterator<'a>
  for Map<'a, T, ItemIn, ItemOut>
where
  T: ConnectionIterator<'a, Item = ItemIn> + ?Sized,
{
  fn current_nesting<'b>(&'b self) -> &'b Nesting {
    self.backing_iterator(|it: &'b Box<T>| it.current_nesting())
  }

  fn finish(mut self: Box<Self>) -> Nesting {
    self.backing_iterator_mut(|it| it.finish())
  }
}

impl<'a, T, ItemIn, ItemOut> Iterator for Map<'a, T, ItemIn, ItemOut>
where
  T: Iterator<Item = ItemIn> + DynClone + ?Sized,
{
  type Item = ItemOut;

  fn next(&mut self) -> Option<Self::Item> {
    let next = self.backing_iterator_mut(|it| it.next())?;
    Some((*self.f)(next))
  }
}

impl<'a, T, ItemIn: 'static, ItemOut: 'static> Clone for Map<'a, T, ItemIn, ItemOut>
where
  T: Iterator<Item = ItemIn> + DynClone + ?Sized,
{
  fn clone(&self) -> Self {
    Self {
      backing_iterator: BackingIterator::Owned(
        self.backing_iterator(|it| dyn_clone::clone_box(&**it)),
      ),
      // backing_iterator: BackingIterator::Owned(dyn_clone::clone_box(&*self.backing_iterator())),
      f: Rc::clone(&self.f),
    }
  }
}

pub fn map<'a, T, ItemIn: 'static, ItemOut: 'static>(
  it: &'a mut Box<T>,
  f: Rc<dyn Fn(ItemIn) -> ItemOut>,
) -> Box<dyn ConnectionIterator<'a, Item = ItemOut> + 'a>
where
  T: ConnectionIterator<'a, Item = ItemIn> + 'a + ?Sized,
{
  // TODO: the problem is that the Map either owns the backing iterator (if it is a clone) or
  // doesn't (in some cases, if it is not a clone). The obvious way to handle these two cases is to
  // use an enum.
  Box::new(Map {
    backing_iterator: BackingIterator::Borrowed(it),
    f: Rc::clone(&f),
  })
}

#[derive(Clone)]
struct VecIterator<Item> {
  v: Vec<Item>,
  pos: usize,
  nesting: Nesting,
}

impl<Item: Clone> Iterator for VecIterator<Item> {
  type Item = Item;

  fn next(&mut self) -> Option<Self::Item> {
    let ret = self.v.get(self.pos)?;
    self.pos += 1;
    Some(ret.clone())
  }
}

impl<'a, Item: Clone> ConnectionIterator<'a> for VecIterator<Item> {
  fn current_nesting(&self) -> &Nesting {
    &self.nesting
  }
  fn finish(mut self: Box<Self>) -> Nesting {
    self.nesting.stop_producer();
    self.nesting
  }
}

pub fn iterator_new<'a, Item: Clone + 'static>(
  mut nesting: Nesting,
  iface: Box<dyn RtorIface>,
  v: Vec<Item>,
) -> Box<dyn ConnectionIterator<'a, Item = Item>> {
  nesting.start_producer(iface);
  Box::new(VecIterator { v, pos: 0, nesting })
}
