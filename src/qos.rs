use napi_derive::napi;
use zenoh as z;

/// The strategy applied when a message has to be routed through a node whose queue is
/// full.
#[napi(string_enum = "snake_case")]
pub enum CongestionControl {
  /// The node may drop the message.
  Drop,
  /// The node waits for the queue to progress.
  Block,
  /// The node waits for the queue to progress, but only for the first message sent with
  /// this strategy; the following ones are dropped.
  BlockFirst,
}

impl From<CongestionControl> for z::qos::CongestionControl {
  fn from(value: CongestionControl) -> Self {
    match value {
      CongestionControl::Drop => Self::Drop,
      CongestionControl::Block => Self::Block,
      CongestionControl::BlockFirst => Self::BlockFirst,
    }
  }
}

impl From<z::qos::CongestionControl> for CongestionControl {
  fn from(value: z::qos::CongestionControl) -> Self {
    match value {
      z::qos::CongestionControl::Drop => Self::Drop,
      z::qos::CongestionControl::Block => Self::Block,
      z::qos::CongestionControl::BlockFirst => Self::BlockFirst,
    }
  }
}

/// The priority of a message.
///
/// If `QoS` is enabled in the session configuration, Zenoh keeps one transmission queue
/// per priority, and services those queues in the order the priorities are listed here,
/// from `RealTime` down to `Background`.
///
/// The default is `Data`.
#[napi(string_enum = "snake_case")]
pub enum Priority {
  RealTime,
  InteractiveHigh,
  InteractiveLow,
  DataHigh,
  Data,
  DataLow,
  Background,
}

impl From<Priority> for z::qos::Priority {
  fn from(value: Priority) -> Self {
    match value {
      Priority::RealTime => Self::RealTime,
      Priority::InteractiveHigh => Self::InteractiveHigh,
      Priority::InteractiveLow => Self::InteractiveLow,
      Priority::DataHigh => Self::DataHigh,
      Priority::Data => Self::Data,
      Priority::DataLow => Self::DataLow,
      Priority::Background => Self::Background,
    }
  }
}

impl From<z::qos::Priority> for Priority {
  fn from(value: z::qos::Priority) -> Self {
    match value {
      z::qos::Priority::RealTime => Self::RealTime,
      z::qos::Priority::InteractiveHigh => Self::InteractiveHigh,
      z::qos::Priority::InteractiveLow => Self::InteractiveLow,
      z::qos::Priority::DataHigh => Self::DataHigh,
      z::qos::Priority::Data => Self::Data,
      z::qos::Priority::DataLow => Self::DataLow,
      z::qos::Priority::Background => Self::Background,
    }
  }
}

/// The reliability requested when routing a message.
///
/// Note: reliability does not trigger any data retransmission on the wire. It is a marker
/// that may be used to select the best link available (e.g. TCP for reliable data and UDP
/// for best effort data).
#[napi(string_enum = "snake_case")]
pub enum Reliability {
  /// Accepts that messages may be lost.
  BestEffort,
  /// Requests that messages be delivered reliably.
  Reliable,
}

impl From<Reliability> for z::qos::Reliability {
  fn from(value: Reliability) -> Self {
    match value {
      Reliability::BestEffort => Self::BestEffort,
      Reliability::Reliable => Self::Reliable,
    }
  }
}

impl From<z::qos::Reliability> for Reliability {
  fn from(value: z::qos::Reliability) -> Self {
    match value {
      z::qos::Reliability::BestEffort => Self::BestEffort,
      z::qos::Reliability::Reliable => Self::Reliable,
    }
  }
}

/// The locality of the entities an operation applies to.
///
/// It restricts subscribers and queryables to receiving from, and publishers and queriers
/// to sending to, only the entities of the given locality.
#[napi(string_enum = "snake_case")]
pub enum Locality {
  /// Only the entities in the same session.
  SessionLocal,
  /// Only the entities that are not in the same session.
  Remote,
  /// Both local and remote entities.
  Any,
}

impl From<Locality> for z::sample::Locality {
  fn from(value: Locality) -> Self {
    match value {
      Locality::SessionLocal => Self::SessionLocal,
      Locality::Remote => Self::Remote,
      Locality::Any => Self::Any,
    }
  }
}
