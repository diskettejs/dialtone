use napi_derive::napi;
use zenoh as z;

#[napi(string_enum)]
pub enum CongestionControl {
  Drop,
  Block,
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

#[napi(string_enum)]
pub enum Reliability {
  BestEffort,
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

#[napi(string_enum)]
pub enum Locality {
  SessionLocal,
  Remote,
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
