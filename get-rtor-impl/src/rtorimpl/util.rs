use irlf_db::ir::Inst;

pub fn require_empty(part: &[Inst]) {
  if !part.is_empty() {
    panic!()
  }
}
