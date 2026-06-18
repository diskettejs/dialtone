//! QoS and addressing enums shared across the session, publisher, and sample
//! APIs. Each is exposed to JS as a string-literal union of its variant names.

use napi_derive::napi;

/// Whether a sample carries a published value or marks a deletion.
#[napi(string_enum)]
pub enum SampleKind {
  /// A value was published.
  Put,
  /// A value was deleted (tombstone).
  Delete,
}

impl From<zenoh::sample::SampleKind> for SampleKind {
  fn from(k: zenoh::sample::SampleKind) -> Self {
    match k {
      zenoh::sample::SampleKind::Put => SampleKind::Put,
      zenoh::sample::SampleKind::Delete => SampleKind::Delete,
    }
  }
}

/// How a message behaves when the network is congested.
#[napi(string_enum)]
pub enum CongestionControl {
  /// Drop the message rather than wait.
  Drop,
  /// Block until the message can be sent.
  Block,
}

impl From<zenoh::qos::CongestionControl> for CongestionControl {
  fn from(c: zenoh::qos::CongestionControl) -> Self {
    match c {
      zenoh::qos::CongestionControl::Drop => CongestionControl::Drop,
      zenoh::qos::CongestionControl::Block => CongestionControl::Block,
    }
  }
}

impl From<CongestionControl> for zenoh::qos::CongestionControl {
  fn from(c: CongestionControl) -> Self {
    match c {
      CongestionControl::Drop => zenoh::qos::CongestionControl::Drop,
      CongestionControl::Block => zenoh::qos::CongestionControl::Block,
    }
  }
}

/// Transmission priority, from highest (`RealTime`) to lowest (`Background`).
#[napi(string_enum)]
pub enum Priority {
  RealTime,
  InteractiveHigh,
  InteractiveLow,
  DataHigh,
  Data,
  DataLow,
  Background,
}

impl From<zenoh::qos::Priority> for Priority {
  fn from(p: zenoh::qos::Priority) -> Self {
    use zenoh::qos::Priority as Z;
    match p {
      Z::RealTime => Priority::RealTime,
      Z::InteractiveHigh => Priority::InteractiveHigh,
      Z::InteractiveLow => Priority::InteractiveLow,
      Z::DataHigh => Priority::DataHigh,
      Z::Data => Priority::Data,
      Z::DataLow => Priority::DataLow,
      Z::Background => Priority::Background,
    }
  }
}

impl From<Priority> for zenoh::qos::Priority {
  fn from(p: Priority) -> Self {
    use zenoh::qos::Priority as Z;
    match p {
      Priority::RealTime => Z::RealTime,
      Priority::InteractiveHigh => Z::InteractiveHigh,
      Priority::InteractiveLow => Z::InteractiveLow,
      Priority::DataHigh => Z::DataHigh,
      Priority::Data => Z::Data,
      Priority::DataLow => Z::DataLow,
      Priority::Background => Z::Background,
    }
  }
}

/// Restricts which subscribers a message can reach, by their location relative to
/// this session.
#[napi(string_enum)]
pub enum Locality {
  /// Only subscribers within this same session.
  SessionLocal,
  /// Only subscribers in other sessions.
  Remote,
  /// Any subscriber whether local or remote.
  Any,
}

impl From<Locality> for zenoh::sample::Locality {
  fn from(l: Locality) -> Self {
    match l {
      Locality::SessionLocal => zenoh::sample::Locality::SessionLocal,
      Locality::Remote => zenoh::sample::Locality::Remote,
      Locality::Any => zenoh::sample::Locality::Any,
    }
  }
}
