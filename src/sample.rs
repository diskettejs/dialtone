use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::session::EntityGlobalId;

/// Restricts which entities a publication reaches (or a subscription accepts),
/// by whether they live in the same session or elsewhere.
#[napi(string_enum)]
pub enum Locality {
  /// Only entities in the same session.
  SessionLocal,
  /// Only remote entities (not in the same session).
  Remote,
  /// Both local and remote entities (the default).
  Any,
}

impl From<Locality> for zenoh::sample::Locality {
  fn from(value: Locality) -> Self {
    match value {
      Locality::SessionLocal => zenoh::sample::Locality::SessionLocal,
      Locality::Remote => zenoh::sample::Locality::Remote,
      Locality::Any => zenoh::sample::Locality::Any,
    }
  }
}

/// Source metadata for a publication: which entity produced the sample and the
/// source's own sequence number for it. Used by advanced pub/sub (e.g. for
/// missing-sample detection); the base primitives just transmit it.
#[napi(object)]
pub struct SourceInfo {
  /// Id of the entity that produced the sample.
  pub source_id: EntityGlobalId,
  /// The source's sequence number for this sample.
  pub source_sn: u32,
}

impl SourceInfo {
  pub(crate) fn to_zenoh(&self) -> Result<zenoh::sample::SourceInfo> {
    Ok(zenoh::sample::SourceInfo::new(
      self.source_id.to_zenoh()?,
      self.source_sn,
    ))
  }
}
