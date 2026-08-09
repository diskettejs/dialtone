use std::time::Duration;

use napi_derive::napi;

use crate::{bytes::*, handlers::*, qos::*, query::*};

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct PublisherPutOptions {
  pub encoding: Option<String>,
  pub attachment: Option<BytesBuffer>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct PublisherDeleteOptions {
  pub attachment: Option<BytesBuffer>,
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
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct SubscriberOptions {
  pub allowed_origin: Option<Locality>,
  pub history: Option<HistoryConfig>,
  pub recovery: Option<napi::Either<PeriodicQueriesRecovery, HeartbeatRecovery>>,
  pub subscriber_detection: Option<bool>,
  pub subscriber_detection_metadata: Option<String>,
  pub query_timeout_ms: Option<f64>,
  #[napi(ts_type = "FifoChannel | RingChannel")]
  pub channel: Option<ChannelHandler<zenoh::sample::Sample>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct PutOptions {
  pub encoding: Option<String>,
  pub congestion_control: Option<CongestionControl>,
  pub priority: Option<Priority>,
  pub express: Option<bool>,
  pub reliability: Option<Reliability>,
  pub allowed_destination: Option<Locality>,
  pub attachment: Option<BytesBuffer>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct DeleteOptions {
  pub congestion_control: Option<CongestionControl>,
  pub priority: Option<Priority>,
  pub express: Option<bool>,
  pub reliability: Option<Reliability>,
  pub allowed_destination: Option<Locality>,
  pub attachment: Option<BytesBuffer>,
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

impl From<PeriodicQueriesRecovery> for zenoh_ext::RecoveryConfig {
  fn from(value: PeriodicQueriesRecovery) -> Self {
    zenoh_ext::RecoveryConfig::<false>::default()
      .periodic_queries(Duration::from_millis(value.period_ms as u64))
  }
}

impl From<HeartbeatRecovery> for zenoh_ext::RecoveryConfig {
  fn from(_value: HeartbeatRecovery) -> Self {
    zenoh_ext::RecoveryConfig::<false>::default().heartbeat()
  }
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct ScoutOptions {
  #[napi(ts_type = "FifoChannel | RingChannel")]
  pub channel: Option<ChannelHandler<zenoh::scouting::Hello>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct MatchingListenerOptions {
  #[napi(ts_type = "FifoChannel | RingChannel")]
  pub channel: Option<ChannelHandler<zenoh::matching::MatchingStatus>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct SampleMissListenerOptions {
  #[napi(ts_type = "FifoChannel | RingChannel")]
  pub channel: Option<ChannelHandler<zenoh_ext::Miss>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct TransportEventsListenerOptions {
  pub history: Option<bool>,
  #[napi(ts_type = "FifoChannel | RingChannel")]
  pub channel: Option<ChannelHandler<zenoh::session::TransportEvent>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct LinkEventsListenerOptions {
  pub history: Option<bool>,
  #[napi(ts_type = "FifoChannel | RingChannel")]
  pub channel: Option<ChannelHandler<zenoh::session::LinkEvent>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct LivelinessSubscriberOptions {
  pub history: Option<bool>,
  #[napi(ts_type = "FifoChannel | RingChannel")]
  pub channel: Option<ChannelHandler<zenoh::sample::Sample>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct LivelinessGetOptions {
  pub timeout: Option<f64>,
  // pub cancellation_token: Option<Instance<CancellationToken>>,
  #[napi(ts_type = "FifoChannel | RingChannel")]
  pub channel: Option<ChannelHandler<zenoh::query::Reply>>,
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
pub struct ReplyOptions {
  pub encoding: Option<String>,
  pub express: Option<bool>,
  pub attachment: Option<BytesBuffer>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct ReplyErrOptions {
  pub encoding: Option<String>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct ReplyDelOptions {
  pub express: Option<bool>,
  pub attachment: Option<BytesBuffer>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct QuerierGetOptions {
  pub parameters: Option<ParametersLike>,
  pub payload: Option<BytesBuffer>,
  pub encoding: Option<String>,
  pub attachment: Option<BytesBuffer>,
  // pub cancellation_token: Option<Instance<CancellationToken>>,
  #[napi(ts_type = "FifoChannel | RingChannel")]
  pub channel: Option<ChannelHandler<zenoh::query::Reply>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct QueryableOptions {
  pub complete: Option<bool>,
  pub allowed_origin: Option<Locality>,
  #[napi(ts_type = "FifoChannel | RingChannel")]
  pub channel: Option<ChannelHandler<zenoh::query::Query>>,
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
pub struct GetOptions {
  pub parameters: Option<ParametersLike>,
  pub target: Option<QueryTarget>,
  pub consolidation: Option<ConsolidationMode>,
  pub congestion_control: Option<CongestionControl>,
  pub priority: Option<Priority>,
  pub express: Option<bool>,
  pub allowed_destination: Option<Locality>,
  pub timeout: Option<f64>,
  pub payload: Option<BytesBuffer>,
  pub encoding: Option<String>,
  pub attachment: Option<BytesBuffer>,
  // pub cancellation_token: Option<ClassInstance<'env, CancellationToken>>,
  #[napi(ts_type = "FifoChannel | RingChannel")]
  pub channel: Option<ChannelHandler<zenoh::query::Reply>>,
}
