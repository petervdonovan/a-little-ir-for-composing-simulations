use crate::iterators::cloneiterator::{self, iterator_new, map};
use crate::iterators::lazyclone::LazyIterClone;
use crate::rtor::{
  trivial_inputs_giver, trivial_inputs_iface_giver, InputsGiver, InputsIface, LevelIterator, Rtor,
  RtorComptime, RtorIface, SetPort,
};
use crate::Db;
use irlf_db::ir::{Inst, LibCtor};
use lf_types::{Level, Net, Side};
use std::cell::Cell;
use std::collections::HashSet;
use std::{any::Any, cell::RefCell, marker::PhantomData, rc::Rc};

#[derive(Clone)]
pub struct FunRtorIface {
  f: Rc<dyn Fn(u64) -> u64>,
}

struct FunRtorComptime {
  downstream: Rc<RefCell<Option<InputsIface>>>,
  level: Rc<Cell<Level>>,
}
struct FunRtor<'db> {
  downstream: Option<InputsGiver<'db>>,
  f: Rc<dyn Fn(u64) -> u64>,
  phantom: PhantomData<&'db u64>,
}

impl FunRtorIface {
  fn new(f: Rc<dyn Fn(u64) -> u64>) -> Self {
    FunRtorIface { f }
  }
}

impl FunRtorComptime {
  fn new() -> Self {
    FunRtorComptime {
      downstream: Rc::new(RefCell::new(None)),
      level: Rc::new(Cell::new(Level(0))), // TODO: check?
    }
  }
}

impl<'db> Rtor<'db> for FunRtor<'db> {
  fn accept(&mut self, side: Side, inputs: InputsGiver<'db>) -> bool {
    if let Side::Right = side {
      self.downstream = Some(inputs);
      false
    } else {
      false
    }
  }

  fn provide(&'db self, side: Side) -> InputsGiver<'db> {
    if let Side::Right = side {
      return Box::new(trivial_inputs_giver);
    }
    Box::new(|| {
      let fclone = self.f.clone();
      Box::new((self.downstream.as_ref().unwrap())().map(move |it| {
        let fcloneclone = fclone.clone();
        let mapped_it = move |sth: &dyn Any| {
          let sth = sth.downcast_ref::<u64>().unwrap();
          #[allow(clippy::redundant_closure_call)]
          let mapped = fcloneclone(*sth);
          (*it)(&mapped)
        };
        let b: SetPort<'db> = Box::new(mapped_it);
        b
      }))
    })
  }

  fn step_forward(&mut self, _distance: u64) -> Option<Net> {
    None
  }

  fn step_down(&mut self) {}

  fn step_up(&mut self) -> Option<Net> {
    None
  }
}

impl RtorComptime for FunRtorComptime {
  fn iterate_levels(&mut self) -> bool {
    false
  }
  fn levels(&self) -> HashSet<Level> {
    HashSet::new() // never notify; fn-like rtors react immediately
  }
  fn accept(&mut self, part: &[Inst], side: Side, inputs: &mut InputsIface) {
    if !part.is_empty() {
      panic!()
    }
    if let Side::Right = side {
      RefCell::replace(self.downstream.as_ref(), Some(inputs.clone()));
      inputs.next(); // ! This assumes that the width of self is 1 !
    }
  }
  fn provide(&self, part: &[Inst], side: Side) -> InputsIface {
    if !part.is_empty() {
      panic!()
    }
    // trivial_inputs_iface_giver()
    if let Side::Right = side {
      trivial_inputs_iface_giver()
    } else {
      let self_level = Rc::clone(&self.level);
      map(
        Box::new(LazyIterClone::new(Rc::clone(&self.downstream))),
        Rc::new(move |it| {
          let self_level = Rc::clone(&self_level);
          Rc::new(move |x| {
            self_level.replace(x);
            it(x)
          })
        }),
      )
    }
  }
}

impl RtorIface for FunRtorIface {
  fn immut_accept<'db>(
    &self,
    db: &'db dyn Db,
    part: &[Inst],
    side: Side,
    inputs_iface: &mut InputsIface,
  ) -> bool {
    todo!()
  }
  fn immut_provide<'db>(
    &self,
    db: &'db dyn Db,
    part: &[Inst],
    side: Side,
    starting_level: Level,
  ) -> LevelIterator {
    iterator_new(vec![starting_level])
  }

  fn n_levels<'db>(&self, db: &'db dyn Db) -> Level {
    Level(0) // no joining of distinct data flows
  }

  fn comptime_realize<'db>(&self, db: &'db dyn Db) -> Box<dyn RtorComptime> {
    todo!()
  }
  fn realize<'db>(
    &self,
    db: &'db dyn Db,
    _inst_time_args: Vec<&'db dyn std::any::Any>,
  ) -> Box<dyn Rtor<'db> + 'db> {
    Box::new(FunRtor {
      downstream: None,
      phantom: PhantomData,
      f: Rc::clone(&self.f),
    })
  }

  fn immut_provide_unique<'db>(
    &self,
    db: &'db dyn Db,
    part: &[Inst],
    side: Side,
    starting_level: Level,
  ) -> HashSet<Level> {
    todo!()
  }

  fn side<'db>(
    &'db self,
    db: &'db dyn Db,
    side: Side,
    part: &[Inst],
  ) -> Box<dyn Iterator<Item = (Level, Box<dyn RtorIface>)>> {
    let cself: Box<dyn RtorIface> = Box::new(self.clone());
    Box::new(vec![(Level(0), cself)].into_iter())
  }
}

pub fn lctor_of<'db>(db: &'db dyn irlf_db::Db, lctor: LibCtor) -> Box<dyn RtorIface + 'db> {
  match lctor.name(db).as_str() {
    "add1" => Box::new(FunRtorIface::new(Rc::new(|x| x + 1))),
    "mul2" => Box::new(FunRtorIface::new(Rc::new(|x| x * 2))),
    _ => panic!(),
  }
}
