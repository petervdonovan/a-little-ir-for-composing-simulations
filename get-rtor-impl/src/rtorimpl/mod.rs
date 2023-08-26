pub mod bifunrtorimpl;
pub mod funrtorimpl;
pub mod srtorimpl;
mod util;

use std::ops::BitOrAssign;

use crate::{
  rtor::RtorIface,
  rtorimpl::{bifunrtorimpl::BiFunRtorIface, funrtorimpl::FunRtorIface},
  Db,
};
use irlf_db::ir::{Ctor, LibCtor};

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
    Ctor::LibCtor(lctor) => lctor_of(db, *lctor),
  }
}

#[salsa::tracked]
pub fn lctor_of(db: &dyn crate::Db, lctor: LibCtor) -> Box<dyn RtorIface> {
  match lctor.name(db).as_str() {
    "add1" => Box::new(FunRtorIface::new(|x| x + 1)),
    "mul2" => Box::new(FunRtorIface::new(|x| x * 2)),
    "sum" => Box::new(BiFunRtorIface::new(|x, y| x + y)),
    "prod" => Box::new(BiFunRtorIface::new(|x, y| x * y)),
    s => panic!("\"{s}\" is not an lctor name"),
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashSet;

  use expect_test::{expect, Expect};
  use irlf_db::from_text;
  use lf_types::{Level, Side};

  use crate::{iterators::nesting::Nesting, GriTestDatabase};

  use super::*;

  fn sort<T: Ord>(v: HashSet<T>) -> Vec<T> {
    let mut v: Vec<_> = v.into_iter().collect();
    v.sort();
    v
  }

  fn shallow_expect(text: &str, expect: Expect) {
    let db = GriTestDatabase::default();
    let (program, _inst2sym) = from_text(text, &db);
    let iface = iface_of(&db, program.main(&db));
    let levels = format!(
      "levels: {:?}\nleft: {:?}\nright: {:?}\nunique_left: {:?}\nunique_right: {:?}",
      sort(iface.levels(&db)),
      iface
        .immut_provide(&db, &[], Side::Left, Level(0), Nesting::default())
        .collect::<Vec<_>>(),
      iface
        .immut_provide(&db, &[], Side::Right, Level(0), Nesting::default())
        .collect::<Vec<_>>(),
      sort(iface.immut_provide_unique(&db, &[], Side::Left, Level(0))),
      sort(iface.immut_provide_unique(&db, &[], Side::Right, Level(0)))
    );
    expect.assert_eq(&levels);
  }

  const BASIC_NO_MERGING: &str = "mul2 0x0 mul2
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
  const MERGING: &str = "add1 0 add1
sum 1 sum
---
---
rtor0 2
  madd1_0 100 = 0
  madd1_2 101 = 0
  ---
  L 100 R 100
  L 101 R 101
  ---
rtor1 3
  sctor0 102 = 2
  msum 103 = 1
  ---
  L 102
  L -
  R 103
  ---
  200 102 103
---
3
";

  #[test]
  fn test0() {
    let text = BASIC_NO_MERGING;
    let expect = expect![[r#"
        levels: [Level(0)]
        left: [Data(Level(0)), Data(Level(0))]
        right: [Data(Level(0)), Data(Level(0))]
        unique_left: [Level(0)]
        unique_right: [Level(0)]"#]];
    shallow_expect(text, expect);
  }
  #[test]
  fn test1() {
    let text = MERGING;
    let expect = expect![[r#"
        levels: [Level(0), Level(1)]
        left: [Data(Level(0)), Data(Level(0))]
        right: [Data(Level(1))]
        unique_left: [Level(0)]
        unique_right: [Level(1)]"#]];
    shallow_expect(text, expect);
  }
}
