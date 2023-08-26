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
