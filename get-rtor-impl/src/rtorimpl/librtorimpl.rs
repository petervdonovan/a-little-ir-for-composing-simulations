use std::{any::Any, marker::PhantomData};

use crate::rtor::{trivial_inputs_giver, InputsGiver, Net, Rtor, SetPort, Side};

macro_rules! fun1rtor {
  ($CtorName: ident, $input_type: ident, $map: expr) => {
    struct $CtorName<'db> {
      downstream: Option<InputsGiver<'db>>,
      phantom: PhantomData<&'db u64>,
    }

    impl<'db> Rtor<'db> for $CtorName<'db> {
      fn new(_inst_time_args: Vec<&'db dyn std::any::Any>) -> Self {
        $CtorName {
          downstream: None,
          phantom: PhantomData,
        }
      }

      fn accept(&mut self, side: Side, inputs: InputsGiver<'db>) -> bool {
        if let Side::Right = side {
          self.downstream = Some(inputs);
          false
        } else {
          false
        }
      }

      fn provide(&'db self, side: crate::rtor::Side) -> InputsGiver<'db> {
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
  };
}

fun1rtor!(Add1, u64, |sth| sth + 1);

fun1rtor!(Mul2, u64, |sth| sth * 2);
