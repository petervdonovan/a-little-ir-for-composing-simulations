use crate::rtor::{
  trivial_inputs_giver, trivial_inputs_iface_giver, InputsGiver, InputsIface, InputsIfaceIterator,
  Rtor, RtorIface, SetPort, ShareLevelLowerBound,
};
use crate::rtorimpl::lazyclone::LazyIterClone;
use irlf_db::ir::LibCtor;
use lf_types::{Net, Side};
use std::{any::Any, marker::PhantomData};

impl<'a> InputsIfaceIterator<'a>
  for LazyIterClone<'a, ShareLevelLowerBound<'a>, dyn InputsIfaceIterator<'a>>
{
}

macro_rules! fun1rtor {
  ($CtorName: ident, $CtorIfaceName: ident, $input_type: ident, $map: expr) => {
    struct $CtorIfaceName<'a> {
      downstream: Option<InputsIface<'a>>,
    }
    struct $CtorName<'db> {
      downstream: Option<InputsGiver<'db>>,
      phantom: PhantomData<&'db u64>,
    }

    impl<'a> Default for $CtorIfaceName<'a> {
      fn default() -> Self {
        $CtorIfaceName { downstream: None }
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
    impl<'a> RtorIface<'a> for $CtorIfaceName<'a> {
      // fn new(_ctor: Ctor, _depth: u32, _comp_time_args: Vec<&'db dyn Any>) -> Self {
      //   $CtorIfaceName {
      //     downstream: None,
      //     phantom: PhantomData,
      //   }
      // }
      fn accept(&mut self, side: Side, inputs: &mut InputsIface<'a>) {
        if let Side::Right = side {
          self.downstream = Some(inputs.clone());
          inputs.next(); // ! This assumes that the width of self is 1 !
        }
      }
      fn provide(&'a self, side: Side) -> InputsIface<'a> {
        if let Side::Right = side {
          trivial_inputs_iface_giver()
        } else {
          Box::new(LazyIterClone::new(&self.downstream))
        }
      }
      fn iterate_levels(&mut self) -> bool {
        false
      }
      fn levels(&self) -> Vec<u32> {
        vec![] // never notify; fn-like rtors react immediately
      }
      // fn realize<'db>(&self, _inst_time_args: Vec<&'db dyn std::any::Any>) -> Box<dyn Rtor + 'db> {
      fn realize<'db>(
        &self,
        _inst_time_args: Vec<&'db dyn std::any::Any>,
      ) -> Box<dyn Rtor<'db> + 'db> {
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

pub fn lctor_of<'db>(db: &'db dyn irlf_db::Db, lctor: LibCtor) -> Box<dyn RtorIface + 'db> {
  match lctor.name(db).as_str() {
    "add1" => Box::<Add1Iface<'_>>::default(),
    "mul2" => Box::<Mul2Iface<'_>>::default(),
    _ => panic!(),
  }
}
