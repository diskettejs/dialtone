use napi_derive::napi;
use zenoh::sample as zsample;

use crate::{bytes::*, config::*, encoding::*, key_expr::*, qos::*, time::*};

#[napi(string_enum)]
pub enum SampleKind {
  Put,
  Delete,
}

impl From<zsample::SampleKind> for SampleKind {
  fn from(kind: zsample::SampleKind) -> Self {
    match kind {
      zsample::SampleKind::Put => SampleKind::Put,
      zsample::SampleKind::Delete => SampleKind::Delete,
    }
  }
}

impl From<SampleKind> for zsample::SampleKind {
  fn from(kind: SampleKind) -> Self {
    match kind {
      SampleKind::Put => zsample::SampleKind::Put,
      SampleKind::Delete => zsample::SampleKind::Delete,
    }
  }
}

#[napi]
pub struct Sample {
  inner: zsample::SampleFields,
}

impl Sample {
  pub(crate) fn new(zsample: zsample::Sample) -> Self {
    let fields: zsample::SampleFields = zsample.into();
    Sample { inner: fields }
  }
}

impl From<zsample::Sample> for Sample {
  fn from(zsample: zsample::Sample) -> Self {
    Sample::new(zsample)
  }
}

#[napi]
impl Sample {
  #[napi(getter)]
  pub fn payload(&self) -> Bytes {
    self.inner.payload.clone().into()
  }

  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    self.inner.key_expr.clone().into()
  }

  #[napi(getter)]
  pub fn kind(&self) -> SampleKind {
    self.inner.kind.into()
  }

  #[napi(getter)]
  pub fn encoding(&self) -> Encoding {
    self.inner.encoding.clone().into()
  }

  #[napi(getter)]
  pub fn timestamp(&self) -> Option<Timestamp> {
    self.inner.timestamp.map(Timestamp::from)
  }

  #[napi(getter)]
  pub fn express(&self) -> bool {
    self.inner.express
  }

  #[napi(getter)]
  pub fn priority(&self) -> Priority {
    self.inner.priority.into()
  }

  #[napi(getter)]
  pub fn congestion_control(&self) -> CongestionControl {
    self.inner.congestion_control.into()
  }

  #[napi(getter)]
  pub fn reliability(&self) -> Reliability {
    self.inner.reliability.into()
  }

  #[napi(getter)]
  pub fn attachment(&self) -> Option<Bytes> {
    self.inner.attachment.clone().map(Bytes::from)
  }

  #[napi(getter)]
  pub fn source_info(&self) -> Option<SourceInfo> {
    self.inner.source_info.clone().map(SourceInfo::from)
  }
}

#[napi]
pub struct SourceInfo {
  inner: zsample::SourceInfo,
}

impl From<zsample::SourceInfo> for SourceInfo {
  fn from(inner: zsample::SourceInfo) -> Self {
    Self { inner }
  }
}

impl From<&SourceInfo> for zsample::SourceInfo {
  fn from(value: &SourceInfo) -> Self {
    value.inner.clone()
  }
}

#[napi]
impl SourceInfo {
  #[napi(constructor)]
  pub fn new(source_id: &EntityGlobalId, source_sn: u32) -> Self {
    Self {
      inner: zsample::SourceInfo::new(source_id.into(), source_sn),
    }
  }

  #[napi(getter)]
  pub fn source_id(&self) -> EntityGlobalId {
    (*self.inner.source_id()).into()
  }

  #[napi(getter)]
  pub fn source_sn(&self) -> u32 {
    self.inner.source_sn()
  }
}
