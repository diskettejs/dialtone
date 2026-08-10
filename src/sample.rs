use derive_more::From;
use napi_derive::napi;

use crate::{
  bytes::{Bytes, Encoding},
  key_expr::KeyExpr,
  qos::{CongestionControl, Priority, Reliability},
  time::Timestamp,
};

#[napi(string_enum)]
pub enum SampleKind {
  Put,
  Delete,
}

impl From<zenoh::sample::SampleKind> for SampleKind {
  fn from(value: zenoh::sample::SampleKind) -> Self {
    match value {
      zenoh::sample::SampleKind::Delete => Self::Delete,
      zenoh::sample::SampleKind::Put => Self::Put,
    }
  }
}

#[napi]
#[derive(From)]
pub struct Sample(zenoh::sample::Sample);

#[napi]
impl Sample {
  #[napi(getter)]
  pub fn payload(&self) -> Bytes {
    self.0.payload().clone().into()
  }

  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    self.0.key_expr().clone().into()
  }

  #[napi(getter)]
  pub fn kind(&self) -> SampleKind {
    self.0.kind().into()
  }

  #[napi(getter)]
  pub fn encoding(&self) -> Encoding {
    self.0.encoding().clone().into()
  }

  #[napi(getter)]
  pub fn timestamp(&self) -> Option<Timestamp> {
    self.0.timestamp().cloned().map(Into::into)
  }

  #[napi(getter)]
  pub fn express(&self) -> bool {
    self.0.express()
  }

  #[napi(getter)]
  pub fn priority(&self) -> Priority {
    self.0.priority().into()
  }

  #[napi(getter)]
  pub fn congestion_control(&self) -> CongestionControl {
    self.0.congestion_control().into()
  }

  #[napi(getter)]
  pub fn reliability(&self) -> Reliability {
    self.0.reliability().into()
  }

  #[napi(getter)]
  pub fn attachment(&self) -> Option<Bytes> {
    self.0.attachment().cloned().map(Into::into)
  }
}
