mod librtorimpl;
mod srtorimpl;

use crate::rtor::RtorIface;
use irlf_db::ir::Ctor;

pub fn iface_of<'db>(db: &'db dyn irlf_db::Db, ctor: &Ctor) -> Box<dyn RtorIface<'db> + 'db> {
  match ctor {
    Ctor::StructlikeCtor(sctor) => {
      Box::new(crate::rtorimpl::srtorimpl::SrtorIface::new(db, *sctor))
    }
    Ctor::BinaryCtor(_) => todo!(),
    Ctor::LibCtor(lctor) => crate::rtorimpl::librtorimpl::lctor_of(db, *lctor),
  }
}
