pub mod librtorimpl;
pub mod srtorimpl;

use std::ops::BitOrAssign;

use crate::{rtor::RtorIface, Db};
use irlf_db::ir::Ctor;

#[derive(PartialEq, Eq)]
pub enum FixpointingStatus {
  Changed,
  Unchanged,
}

impl BitOrAssign for FixpointingStatus {
  fn bitor_assign(&mut self, rhs: Self) {
    if rhs == FixpointingStatus::Changed {
      *self = FixpointingStatus::Changed;
    }
  }
}

pub fn iface_of<'db>(db: &'db dyn Db, ctor: &Ctor) -> Box<dyn RtorIface + 'db> {
  match ctor {
    Ctor::StructlikeCtor(sctor) => crate::rtorimpl::srtorimpl::srtor_of(db, *sctor),
    Ctor::BinaryCtor(_) => todo!(),
    Ctor::LibCtor(lctor) => crate::rtorimpl::librtorimpl::lctor_of(db, *lctor),
  }
}

#[cfg(test)]
mod tests {
  use expect_test::expect;
  use irlf_db::from_text;

  use crate::GriTestDatabase;

  use super::{librtorimpl::lctor_of, *};

  #[test]
  fn test() {
    let text = "mul2 0x0 mul2
add1 0x1 add1
---
---
rtor0 0x2
  myadd1 100 = 0x1
  mymul2 101 = 0x0
  ---
  L 100
  R 100
  L 101
  R 101
  ---
rtor1 0x3
  mysctor0 102 = 0x2
  mysctor1 103 = 0x2
  ---
  L 102
  R 103
  ---
  200 102 103
---
0x3
";
    let db = GriTestDatabase::default();
    let (program, _inst2sym) = from_text(text, &db);
    let iface = iface_of(&db, program.main(&db));
    let levels = format!("{:?}", iface.levels(&db));
    let expected = expect![[]];
    expected.assert_eq(&levels);
  }
}
