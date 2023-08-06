use crate::rtor::{
  trivial_inputs_giver, trivial_inputs_iface_giver, InputsGiver, InputsIfaceGiver, Rtor, RtorIface,
  SetPort, ShareLevelLowerBound,
};
use irlf_db::ir::{Ctor, LibCtor};
use lf_types::{Net, Side};
use std::{any::Any, marker::PhantomData};

macro_rules! fun1rtor {
  ($CtorName: ident, $CtorIfaceName: ident, $input_type: ident, $map: expr) => {
    struct $CtorIfaceName<'db> {
      downstream: Option<InputsIfaceGiver<'db>>,
      phantom: PhantomData<&'db u64>,
    }
    struct $CtorName<'db> {
      downstream: Option<InputsGiver<'db>>,
      phantom: PhantomData<&'db u64>,
    }

    impl<'db> Default for $CtorIfaceName<'db> {
      fn default() -> Self {
        $CtorIfaceName {
          downstream: None,
          phantom: PhantomData,
        }
      }
    }

    impl<'db> Rtor<'db> for $CtorName<'db> {
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
          Box::new((self.downstream.as_ref().unwrap())().map(|it| {
            let mapped_it = move |sth: &dyn Any| {
              let sth = sth.downcast_ref::<u64>().unwrap();
              #[allow(clippy::redundant_closure_call)]
              let mapped = ($map)(sth);
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
    impl<'db> RtorIface<'db> for $CtorIfaceName<'db> {
      // fn new(_ctor: Ctor, _depth: u32, _comp_time_args: Vec<&'db dyn Any>) -> Self {
      //   $CtorIfaceName {
      //     downstream: None,
      //     phantom: PhantomData,
      //   }
      // }
      fn accept(&mut self, side: Side, inputs: InputsIfaceGiver<'db>) {
        if let Side::Right = side {
          self.downstream = Some(inputs);
        }
      }
      fn provide(&'db self, side: Side) -> InputsIfaceGiver<'db> {
        if let Side::Right = side {
          return Box::new(trivial_inputs_iface_giver);
        }
        Box::new(|| {
          Box::new((self.downstream.as_ref().unwrap())().map(|it| {
            let mapped_it = move |sth: u32| (*it)(sth);
            let b: ShareLevelLowerBound<'db> = Box::new(mapped_it);
            b
          }))
        })
      }
      fn iterate_levels(&mut self) -> bool {
        false
      }
      fn levels(&self) -> Vec<u32> {
        vec![]
      }
      fn realize(&self, _inst_time_args: Vec<&'db dyn std::any::Any>) -> Box<dyn Rtor + '_> {
        Box::new($CtorName {
          downstream: None,
          phantom: PhantomData,
        })
      }
    }
  };
}

fun1rtor!(Add1, Add1Iface, u64, |sth| sth + 1);

fun1rtor!(Mul2, Mul2Iface, u64, |sth| sth * 2);

pub fn lctor_of<'db>(db: &dyn irlf_db::Db, lctor: LibCtor) -> Box<dyn RtorIface<'db>> {
  let add1: Box<Add1Iface<'static>> = Box::new(Add1Iface::default());
  match lctor.name(db).as_str() {
    "add1" => add1,
    "mul2" => add1, // Box::new(Mul2Iface::default()),
    _ => panic!(),
  }
}
