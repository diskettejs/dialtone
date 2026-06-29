use std::time::Duration;

use napi::{Either, bindgen_prelude::*};
use napi_derive::napi;

use crate::{cancellation::*, channels::*, qos::*, query::*, sample::*, time::*};

#[napi]
pub type ChannelArg<'a> = Either<ClassInstance<'a, FifoChannel>, ClassInstance<'a, RingChannel>>;

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct PublisherPutOptions<'a> {
  pub encoding: Option<String>,
  pub timestamp: Option<ClassInstance<'a, Timestamp>>,
  pub attachment: Option<Uint8Array>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct PublisherDeleteOptions<'a> {
  pub timestamp: Option<ClassInstance<'a, Timestamp>>,
  pub attachment: Option<Uint8Array>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct PublisherOptions {
  pub encoding: Option<String>,
  pub congestion_control: Option<CongestionControl>,
  pub priority: Option<Priority>,
  pub express: Option<bool>,
  pub reliability: Option<Reliability>,
  pub allowed_destination: Option<Locality>,
  pub cache: Option<CacheConfig>,
  pub sample_miss_detection: Option<MissDetectionConfig>,
  pub publisher_detection: Option<bool>,
  pub publisher_detection_metadata: Option<String>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct SubscriberOptions<'a> {
  pub allowed_origin: Option<Locality>,
  pub history: Option<HistoryConfig>,
  pub recovery: Option<Either<PeriodicQueriesRecovery, HeartbeatRecovery>>,
  pub subscriber_detection: Option<bool>,
  pub subscriber_detection_metadata: Option<String>,
  pub query_timeout_ms: Option<f64>,
  pub channel: Option<ChannelArg<'a>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct PutOptions<'a> {
  pub encoding: Option<String>,
  pub congestion_control: Option<CongestionControl>,
  pub priority: Option<Priority>,
  pub express: Option<bool>,
  pub reliability: Option<Reliability>,
  pub allowed_destination: Option<Locality>,
  pub timestamp: Option<ClassInstance<'a, Timestamp>>,
  pub attachment: Option<Uint8Array>,
  pub source_info: Option<ClassInstance<'a, SourceInfo>>,
}

#[napi(object, object_to_js = false)]
pub struct DeleteOptions<'a> {
  pub congestion_control: Option<CongestionControl>,
  pub priority: Option<Priority>,
  pub express: Option<bool>,
  pub reliability: Option<Reliability>,
  pub allowed_destination: Option<Locality>,
  pub timestamp: Option<ClassInstance<'a, Timestamp>>,
  pub attachment: Option<Uint8Array>,
  pub source_info: Option<ClassInstance<'a, SourceInfo>>,
}

#[napi(object, object_to_js = false)]
pub struct HistoryConfig {
  pub detect_late_publishers: Option<bool>,
  pub max_samples: Option<u32>,
  pub max_age_secs: Option<f64>,
}

impl From<HistoryConfig> for zenoh_ext::HistoryConfig {
  fn from(value: HistoryConfig) -> Self {
    let mut config = zenoh_ext::HistoryConfig::default();
    if value.detect_late_publishers == Some(true) {
      config = config.detect_late_publishers();
    }
    if let Some(max_samples) = value.max_samples {
      config = config.max_samples(max_samples as usize);
    }
    if let Some(max_age_secs) = value.max_age_secs {
      config = config.max_age(max_age_secs);
    }
    config
  }
}

#[napi(string_enum)]
pub enum PeriodicQueriesMode {
  PeriodicQueries,
}

#[napi(string_enum)]
pub enum HeartbeatMode {
  Heartbeat,
}

#[napi(object, object_to_js = false)]
pub struct PeriodicQueriesRecovery {
  pub mode: PeriodicQueriesMode,
  pub period_ms: u32,
}

#[napi(object, object_to_js = false)]
pub struct HeartbeatRecovery {
  pub mode: HeartbeatMode,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct ScoutOptions<'a> {
  pub channel: Option<ChannelArg<'a>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct MatchingListenerOptions<'a> {
  pub channel: Option<ChannelArg<'a>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct SampleMissListenerOptions<'a> {
  pub channel: Option<ChannelArg<'a>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct LivelinessSubscriberOptions<'a> {
  pub history: Option<bool>,
  pub channel: Option<ChannelArg<'a>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct LivelinessGetOptions<'a> {
  pub timeout: Option<f64>,
  pub cancellation_token: Option<ClassInstance<'a, CancellationToken>>,
  pub channel: Option<ChannelArg<'a>>,
}

#[napi(object, object_to_js = false)]
pub struct HeartbeatConfig {
  pub period_ms: u32,
  pub sporadic: Option<bool>,
}

#[napi(object, object_to_js = false)]
pub struct MissDetectionConfig {
  pub heartbeat: Option<HeartbeatConfig>,
}

impl From<MissDetectionConfig> for zenoh_ext::MissDetectionConfig {
  fn from(value: MissDetectionConfig) -> Self {
    let mut config = zenoh_ext::MissDetectionConfig::default();
    if let Some(heartbeat) = value.heartbeat {
      let period = Duration::from_millis(heartbeat.period_ms as u64);
      config = if heartbeat.sporadic == Some(true) {
        config.sporadic_heartbeat(period)
      } else {
        config.heartbeat(period)
      };
    }
    config
  }
}

#[napi(object, object_to_js = false)]
pub struct CacheConfig {
  pub max_samples: Option<u32>,
  pub replies_config: Option<RepliesConfig>,
}

impl From<CacheConfig> for zenoh_ext::CacheConfig {
  fn from(value: CacheConfig) -> Self {
    let mut config = zenoh_ext::CacheConfig::default();
    if let Some(max_samples) = value.max_samples {
      config = config.max_samples(max_samples as usize);
    }
    if let Some(replies_config) = value.replies_config {
      config = config.replies_config(replies_config.into());
    }
    config
  }
}

#[napi(object)]
pub struct RepliesConfig {
  pub priority: Option<Priority>,
  pub congestion_control: Option<CongestionControl>,
  pub express: Option<bool>,
}

impl From<RepliesConfig> for zenoh_ext::RepliesConfig {
  fn from(value: RepliesConfig) -> Self {
    let mut config = zenoh_ext::RepliesConfig::default();
    if let Some(priority) = value.priority {
      config = config.priority(priority.into());
    }
    if let Some(congestion_control) = value.congestion_control {
      config = config.congestion_control(congestion_control.into());
    }
    if let Some(express) = value.express {
      config = config.express(express);
    }
    config
  }
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct ReplyOptions<'a> {
  pub encoding: Option<String>,
  pub express: Option<bool>,
  pub timestamp: Option<ClassInstance<'a, Timestamp>>,
  pub attachment: Option<Uint8Array>,
  pub source_info: Option<ClassInstance<'a, SourceInfo>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct ReplyErrOptions {
  pub encoding: Option<String>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct ReplyDelOptions<'a> {
  pub express: Option<bool>,
  pub timestamp: Option<ClassInstance<'a, Timestamp>>,
  pub attachment: Option<Uint8Array>,
  pub source_info: Option<ClassInstance<'a, SourceInfo>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct QuerierGetOptions<'a> {
  pub parameters: Option<ClassInstance<'a, Parameters>>,
  pub payload: Option<Uint8Array>,
  pub encoding: Option<String>,
  pub attachment: Option<Uint8Array>,
  pub source_info: Option<ClassInstance<'a, SourceInfo>>,
  pub cancellation_token: Option<ClassInstance<'a, CancellationToken>>,
  pub channel: Option<ChannelArg<'a>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct QueryableOptions<'a> {
  pub complete: Option<bool>,
  pub allowed_origin: Option<Locality>,
  pub channel: Option<ChannelArg<'a>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct QuerierOptions {
  pub target: Option<QueryTarget>,
  pub consolidation: Option<ConsolidationMode>,
  pub congestion_control: Option<CongestionControl>,
  pub priority: Option<Priority>,
  pub express: Option<bool>,
  pub allowed_destination: Option<Locality>,
  pub timeout: Option<f64>,
  pub accept_replies: Option<ReplyKeyExpr>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct GetOptions<'a> {
  pub parameters: Option<ClassInstance<'a, Parameters>>,
  pub target: Option<QueryTarget>,
  pub consolidation: Option<ConsolidationMode>,
  pub congestion_control: Option<CongestionControl>,
  pub priority: Option<Priority>,
  pub express: Option<bool>,
  pub allowed_destination: Option<Locality>,
  pub timeout: Option<f64>,
  pub payload: Option<Uint8Array>,
  pub encoding: Option<String>,
  pub attachment: Option<Uint8Array>,
  pub source_info: Option<ClassInstance<'a, SourceInfo>>,
  pub cancellation_token: Option<ClassInstance<'a, CancellationToken>>,
  pub channel: Option<ChannelArg<'a>>,
}
