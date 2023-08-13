#![feature(trait_upcasting)]
#![feature(let_chains)]
mod iterators;
pub mod rtor;
mod rtorimpl;

#[salsa::jar(db=Db)]
pub struct Jar(
  crate::rtorimpl::srtorimpl::SrtorIface,
  // crate::rtorimpl::librtorimpl::FunRtorIface,
);

pub trait Db: salsa::DbWithJar<Jar> + irlf_db::Db {}

impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> + irlf_db::Db {}
