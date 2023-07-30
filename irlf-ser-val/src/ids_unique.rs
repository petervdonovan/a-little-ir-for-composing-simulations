use std::collections::HashSet;

use irlf_ser::ir_serializable::{Ctor, CtorId, InstId, Program};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
  #[error("IDs cannot be used twice.")]
  DuplicateId,
}

/// Check that all IDs used (program-wide) are unique.
///
/// ## Errors
/// Validation error if an ID appears twice.
pub fn ids_unique(p: &Program) -> Result<(), ValidationError> {
  let mut ids = HashSet::new();
  for CtorId(key) in p.ctors.keys() {
    ids.insert(key);
  }
  for ctor in p.ctors.values() {
    if let Ctor::StructlikeCtor(sctor) = ctor {
      for InstId(id) in sctor.insts.keys() {
        if ids.contains(id) {
          return Err(ValidationError::DuplicateId);
        }
        ids.insert(id);
      }
    }
  }
  Ok(())
}
