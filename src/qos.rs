use napi_derive::napi;
use zenoh::{qos as zqos, sample as zsample};

use crate::macros::enum_mapper;

enum_mapper!(zqos::CongestionControl: Drop, Block, BlockFirst);

enum_mapper!(
  zqos::Priority: RealTime,
  InteractiveHigh,
  InteractiveLow,
  DataHigh,
  Data,
  DataLow,
  Background
);

enum_mapper!(zqos::Reliability: BestEffort, Reliable);

enum_mapper!(zsample::Locality: SessionLocal, Remote, Any);
