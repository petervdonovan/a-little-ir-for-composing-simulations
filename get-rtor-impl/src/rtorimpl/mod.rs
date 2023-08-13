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
    Ctor::StructlikeCtor(sctor) => {
      Box::new(crate::rtorimpl::srtorimpl::SrtorIface::new(db, *sctor))
    }
    Ctor::BinaryCtor(_) => todo!(),
    Ctor::LibCtor(lctor) => crate::rtorimpl::librtorimpl::lctor_of(db, *lctor),
  }
}
