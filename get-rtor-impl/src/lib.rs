#![feature(trait_upcasting)]
#![feature(let_chains)]
#![feature(fn_traits)]
mod iterators;
pub mod rtor;
mod rtorimpl;

#[salsa::jar(db=Db)]
pub struct Jar(
  crate::rtorimpl::srtorimpl::SrtorIface,
  // crate::rtorimpl::librtorimpl::FunRtorIface,
  rtorimpl::lctor_of,
  rtorimpl::srtorimpl::srtor_of,
);

#[derive(Default)]
#[salsa::db(crate::Jar, irlf_db::Jar)]
pub(crate) struct GriTestDatabase {
  storage: salsa::Storage<Self>,
}
impl salsa::Database for GriTestDatabase {}
pub trait Db: salsa::DbWithJar<Jar> + irlf_db::Db {}
impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> + salsa::DbWithJar<irlf_db::Jar> {}
