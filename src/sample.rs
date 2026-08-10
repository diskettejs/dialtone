use derive_more::From;
use napi_derive::napi;

use crate::{
  bytes::{Bytes, Encoding},
  key_expr::KeyExpr,
  qos::{CongestionControl, Priority, Reliability},
  time::Timestamp,
};

/// The kind of operation a {@link Sample} was issued by.
#[napi(string_enum)]
pub enum SampleKind {
  /// The sample was issued by a put.
  Put,
  /// The sample was issued by a delete.
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

/// The data unit delivered to a subscriber or carried by a successful {@link Reply}.
///
/// It holds the payload along with all the metadata associated with the data.
#[napi]
#[derive(From)]
pub struct Sample(zenoh::sample::Sample);

#[napi]
impl Sample {
  /// The payload of this sample.
  #[napi(getter)]
  pub fn payload(&self) -> Bytes {
    self.0.payload().clone().into()
  }

  /// The key expression this sample was published on.
  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    self.0.key_expr().clone().into()
  }

  /// Whether this sample was issued by a put or by a delete.
  #[napi(getter)]
  pub fn kind(&self) -> SampleKind {
    self.0.kind().into()
  }

  /// The encoding of {@link Sample.payload}.
  #[napi(getter)]
  pub fn encoding(&self) -> Encoding {
    self.0.encoding().clone().into()
  }

  /// The timestamp of this sample, or `null` if it carries none.
  #[napi(getter)]
  pub fn timestamp(&self) -> Option<Timestamp> {
    self.0.timestamp().copied().map(Into::into)
  }

  /// Whether this sample was sent without batching, which usually reduces latency.
  #[napi(getter)]
  pub fn express(&self) -> bool {
    self.0.express()
  }

  /// The priority applied when routing this sample.
  #[napi(getter)]
  pub fn priority(&self) -> Priority {
    self.0.priority().into()
  }

  /// The congestion control applied when routing this sample.
  #[napi(getter)]
  pub fn congestion_control(&self) -> CongestionControl {
    self.0.congestion_control().into()
  }

  /// The reliability applied when routing this sample.
  #[napi(getter)]
  pub fn reliability(&self) -> Reliability {
    self.0.reliability().into()
  }

  /// The arbitrary user-defined data sent alongside the payload, or `null` if there is
  /// none.
  #[napi(getter)]
  pub fn attachment(&self) -> Option<Bytes> {
    self.0.attachment().cloned().map(Into::into)
  }
}
