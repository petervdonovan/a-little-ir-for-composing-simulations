use crate::iterators::cloneiterator::{map, CloneIterator};
use crate::iterators::lazyclone::LazyIterClone;
use crate::rtor::{
  trivial_inputs_giver, trivial_inputs_iface_giver, InputsGiver, InputsIface, LevelIterator, Rtor,
  RtorIface, SetPort, ShareLevelLowerBound,
};
use irlf_db::ir::{Inst, LibCtor};
use lf_types::{Net, Side};
use std::{any::Any, cell::RefCell, marker::PhantomData, rc::Rc};

// impl<'a> CloneIterator<ShareLevelLowerBound<'a>>
//   for LazyIterClone<'a, ShareLevelLowerBound<'a>, dyn CloneIterator<ShareLevelLowerBound<'a>>>
// {
// }

// macro_rules! fun1rtor {
//   ($CtorName: ident, $CtorIfaceName: ident, $input_type: ident, $map: expr) => {
struct FunRtorIface {
  downstream: Rc<RefCell<Option<InputsIface>>>,
  f: Rc<dyn Fn(u64) -> u64>,
}
struct FunRtor<'db> {
  downstream: Option<InputsGiver<'db>>,
  f: Rc<dyn Fn(u64) -> u64>,
  phantom: PhantomData<&'db u64>,
}

impl<'a> FunRtorIface {
  fn new(f: Rc<dyn Fn(u64) -> u64>) -> Self {
    FunRtorIface {
      downstream: Rc::new(RefCell::new(None)),
      f,
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
impl<'a> RtorIface<'a> for FunRtorIface {
  fn accept(&mut self, part: &[Inst], side: Side, inputs: &mut InputsIface) {
    if part.len() > 0 {
      panic!()
    }
    if let Side::Right = side {
      RefCell::replace(self.downstream.as_ref(), Some(inputs.clone()));
      inputs.next(); // ! This assumes that the width of self is 1 !
    }
  }
  fn provide(&self, part: &[Inst], side: Side) -> InputsIface {
    if part.len() > 0 {
      panic!()
    }
    // trivial_inputs_iface_giver()
    if let Side::Right = side {
      trivial_inputs_iface_giver()
    } else {
      // Box::new(map(Box::new(LazyIterClone::new(&self.downstream)), |it| {
      //   // self.level = it;
      //   it
      // }))
      Box::new(LazyIterClone::new(Rc::clone(&self.downstream)))
    }
  }
  fn iterate_levels(&mut self) -> bool {
    false
  }
  fn levels(&self) -> Vec<u32> {
    vec![] // never notify; fn-like rtors react immediately
  }
  fn in_levels(&self) -> Box<dyn LevelIterator> {
    todo!()
  }
  fn out_levels(&self) -> Box<dyn LevelIterator> {
    todo!()
  }
  // fn realize<'db>(&self, _inst_time_args: Vec<&'db dyn std::any::Any>) -> Box<dyn Rtor + 'db> {
  fn realize<'db>(&self, _inst_time_args: Vec<&'db dyn std::any::Any>) -> Box<dyn Rtor<'db> + 'db> {
    Box::new(FunRtor {
      downstream: None,
      phantom: PhantomData,
      f: Rc::clone(&self.f),
    })
  }
}
//   };
// }

// fun1rtor!(Add1, Add1Iface, u64, |sth| sth + 1);

// fun1rtor!(Mul2, Mul2Iface, u64, |sth| sth * 2);

pub fn lctor_of<'db>(db: &'db dyn irlf_db::Db, lctor: LibCtor) -> Box<dyn RtorIface + 'db> {
  match lctor.name(db).as_str() {
    "add1" => Box::new(FunRtorIface::new(Rc::new(|x| x + 1))),
    "mul2" => Box::new(FunRtorIface::new(Rc::new(|x| x * 2))),
    _ => panic!(),
  }
}
