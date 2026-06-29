use napi_derive::napi;
use zenoh::{qos as zqos, sample as zsample};

#[napi(string_enum)]
pub enum CongestionControl {
  Drop,
  Block,
  BlockFirst,
}

impl From<CongestionControl> for zqos::CongestionControl {
  fn from(value: CongestionControl) -> Self {
    match value {
      CongestionControl::Drop => Self::Drop,
      CongestionControl::Block => Self::Block,
      CongestionControl::BlockFirst => Self::BlockFirst,
    }
  }
}

impl From<zqos::CongestionControl> for CongestionControl {
  fn from(value: zqos::CongestionControl) -> Self {
    match value {
      zqos::CongestionControl::Drop => Self::Drop,
      zqos::CongestionControl::Block => Self::Block,
      zqos::CongestionControl::BlockFirst => Self::BlockFirst,
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

impl From<Priority> for zqos::Priority {
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

impl From<zqos::Priority> for Priority {
  fn from(value: zqos::Priority) -> Self {
    match value {
      zqos::Priority::RealTime => Self::RealTime,
      zqos::Priority::InteractiveHigh => Self::InteractiveHigh,
      zqos::Priority::InteractiveLow => Self::InteractiveLow,
      zqos::Priority::DataHigh => Self::DataHigh,
      zqos::Priority::Data => Self::Data,
      zqos::Priority::DataLow => Self::DataLow,
      zqos::Priority::Background => Self::Background,
    }
  }
}

#[napi(string_enum)]
pub enum Reliability {
  BestEffort,
  Reliable,
}

impl From<Reliability> for zqos::Reliability {
  fn from(value: Reliability) -> Self {
    match value {
      Reliability::BestEffort => Self::BestEffort,
      Reliability::Reliable => Self::Reliable,
    }
  }
}

impl From<zqos::Reliability> for Reliability {
  fn from(value: zqos::Reliability) -> Self {
    match value {
      zqos::Reliability::BestEffort => Self::BestEffort,
      zqos::Reliability::Reliable => Self::Reliable,
    }
  }
}

#[napi(string_enum)]
pub enum Locality {
  SessionLocal,
  Remote,
  Any,
}

impl From<Locality> for zsample::Locality {
  fn from(value: Locality) -> Self {
    match value {
      Locality::SessionLocal => Self::SessionLocal,
      Locality::Remote => Self::Remote,
      Locality::Any => Self::Any,
    }
  }
}

impl From<zsample::Locality> for Locality {
  fn from(value: zsample::Locality) -> Self {
    match value {
      zsample::Locality::SessionLocal => Self::SessionLocal,
      zsample::Locality::Remote => Self::Remote,
      zsample::Locality::Any => Self::Any,
    }
  }
}
