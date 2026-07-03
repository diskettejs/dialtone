use std::time::Duration;

use napi_derive::napi;

use crate::{
  bytes::*, cancellation::*, handlers::*, qos::*, query::*, sample::*, time::*, utils::*,
};

/// Identity conversions: primitives passed straight to a setter, plus `Duration` values that
/// `duration_ms` has already produced and that pass back through `build!`.
macro_rules! identity {
  ($($ty:ty),* $(,)?) => {$(
    impl IntoZenoh for $ty {
      type Into = $ty;
      fn into_zenoh(self) -> $ty {
        self
      }
    }
  )*};
}
identity!(bool, String, Duration);

/// Conversions that defer to an existing `From`/`Into`.
/// - `T => U` converts `self` directly (`self.into()`), for structs with a `From<T> for U`.
/// - `Instance: T => U` converts a borrow of the unwrapped instance (`U::from(self.as_ref())`),
///   for zenoh types whose `From` is implemented on the napi wrapper by reference.
macro_rules! via_from {
  ($($ty:ty => $into:ty),* $(,)?) => {$(
    impl IntoZenoh for $ty {
      type Into = $into;
      fn into_zenoh(self) -> $into {
        self.into()
      }
    }
  )*};
  (Instance: $($ty:ty => $into:ty),* $(,)?) => {$(
    impl IntoZenoh for Instance<$ty> {
      type Into = $into;
      fn into_zenoh(self) -> $into {
        <$into>::from(self.as_ref())
      }
    }
  )*};
}
via_from!(
  HistoryConfig => zenoh_ext::HistoryConfig,
  CacheConfig => zenoh_ext::CacheConfig,
  MissDetectionConfig => zenoh_ext::MissDetectionConfig,
);
via_from!(Instance:
  Timestamp => zenoh::time::Timestamp,
  SourceInfo => zenoh::sample::SourceInfo,
  CancellationToken => zenoh::cancellation::CancellationToken,
  Parameters => zenoh::query::Parameters<'static>,
);

impl IntoZenoh for napi::Either<PeriodicQueriesRecovery, HeartbeatRecovery> {
  type Into = zenoh_ext::RecoveryConfig;
  fn into_zenoh(self) -> zenoh_ext::RecoveryConfig {
    match self {
      napi::Either::A(periodic) => periodic.into(),
      napi::Either::B(heartbeat) => heartbeat.into(),
    }
  }
}

/// Converts a JS millisecond timeout into a `Duration`, surfacing invalid values as errors.
pub(crate) fn duration_ms(ms: Option<f64>) -> napi::Result<Option<Duration>> {
  ms.map(|ms| Duration::try_from_secs_f64(ms / 1000.0).map_napi_err())
    .transpose()
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct PublisherPutOptions {
  pub encoding: Option<String>,
  #[napi(ts_type = "Timestamp")]
  pub timestamp: Option<Instance<Timestamp>>,
  pub attachment: Option<Payload>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct PublisherDeleteOptions {
  #[napi(ts_type = "Timestamp")]
  pub timestamp: Option<Instance<Timestamp>>,
  pub attachment: Option<Payload>,
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
  #[napi(ts_type = "Timestamp")]
  pub timestamp: Option<Instance<Timestamp>>,
  pub attachment: Option<Payload>,
  #[napi(ts_type = "SourceInfo")]
  pub source_info: Option<Instance<SourceInfo>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct DeleteOptions {
  pub congestion_control: Option<CongestionControl>,
  pub priority: Option<Priority>,
  pub express: Option<bool>,
  pub reliability: Option<Reliability>,
  pub allowed_destination: Option<Locality>,
  #[napi(ts_type = "Timestamp")]
  pub timestamp: Option<Instance<Timestamp>>,
  pub attachment: Option<Payload>,
  #[napi(ts_type = "SourceInfo")]
  pub source_info: Option<Instance<SourceInfo>>,
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
pub struct LivelinessSubscriberOptions {
  pub history: Option<bool>,
  #[napi(ts_type = "FifoChannel | RingChannel")]
  pub channel: Option<ChannelHandler<zenoh::sample::Sample>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct LivelinessGetOptions {
  pub timeout: Option<f64>,
  #[napi(ts_type = "CancellationToken")]
  pub cancellation_token: Option<Instance<CancellationToken>>,
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
  #[napi(ts_type = "Timestamp")]
  pub timestamp: Option<Instance<Timestamp>>,
  pub attachment: Option<Payload>,
  #[napi(ts_type = "SourceInfo")]
  pub source_info: Option<Instance<SourceInfo>>,
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
  #[napi(ts_type = "Timestamp")]
  pub timestamp: Option<Instance<Timestamp>>,
  pub attachment: Option<Payload>,
  #[napi(ts_type = "SourceInfo")]
  pub source_info: Option<Instance<SourceInfo>>,
}

#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct QuerierGetOptions {
  #[napi(ts_type = "Parameters")]
  pub parameters: Option<Instance<Parameters>>,
  pub payload: Option<Payload>,
  pub encoding: Option<String>,
  pub attachment: Option<Payload>,
  #[napi(ts_type = "SourceInfo")]
  pub source_info: Option<Instance<SourceInfo>>,
  #[napi(ts_type = "CancellationToken")]
  pub cancellation_token: Option<Instance<CancellationToken>>,
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
  #[napi(ts_type = "Parameters")]
  pub parameters: Option<Instance<Parameters>>,
  pub target: Option<QueryTarget>,
  pub consolidation: Option<ConsolidationMode>,
  pub congestion_control: Option<CongestionControl>,
  pub priority: Option<Priority>,
  pub express: Option<bool>,
  pub allowed_destination: Option<Locality>,
  pub timeout: Option<f64>,
  pub payload: Option<Payload>,
  pub encoding: Option<String>,
  pub attachment: Option<Payload>,
  #[napi(ts_type = "SourceInfo")]
  pub source_info: Option<Instance<SourceInfo>>,
  #[napi(ts_type = "CancellationToken")]
  pub cancellation_token: Option<Instance<CancellationToken>>,
  #[napi(ts_type = "FifoChannel | RingChannel")]
  pub channel: Option<ChannelHandler<zenoh::query::Reply>>,
}
