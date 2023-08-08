mod iterators;
pub mod rtor;
mod rtorimpl;

#[salsa::jar(db=Db)]
pub struct Jar();

pub trait Db: salsa::DbWithJar<Jar> + irlf_db::Db {}

impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> + irlf_db::Db {}
