use core::time::Duration;

use napi_derive::napi;

use crate::{
  bytes::BytesBuffer,
  qos::{CongestionControl, Locality, Priority, Reliability},
  query::{ConsolidationMode, ParametersLike, QueryTarget, ReplyKeyExpr},
};

/// Options for a single publication on an existing publisher.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct PublisherPutOptions {
  /// Encoding of this publication's payload.
  ///
  /// Overrides the publisher's default {@link PublisherOptions.encoding} for this
  /// publication only.
  pub encoding: Option<String>,
  /// Arbitrary user-defined data sent alongside the payload.
  pub attachment: Option<BytesBuffer>,
}

/// Options for a single delete on an existing publisher.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct PublisherDeleteOptions {
  /// Arbitrary user-defined data sent alongside the delete.
  pub attachment: Option<BytesBuffer>,
}

/// Options for declaring a publisher.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct PublisherOptions {
  /// Default encoding of the payloads published by this publisher.
  ///
  /// Can be overridden per publication with {@link PublisherPutOptions.encoding}.
  pub encoding: Option<String>,
  /// Congestion control to apply when routing the data.
  pub congestion_control: Option<CongestionControl>,
  /// Priority to apply when routing the data.
  pub priority: Option<Priority>,
  /// When `true`, messages are not batched.
  ///
  /// This usually has a positive impact on latency but a negative impact on throughput.
  pub express: Option<bool>,
  /// Reliability to apply when routing the data.
  ///
  /// Note: reliability does not trigger any data retransmission on the wire. It is a
  /// marker that may be used to select the best link available (e.g. TCP for reliable
  /// data and UDP for best effort data).
  pub reliability: Option<Reliability>,
  /// Restricts the matching subscribers that receive the published data to the ones
  /// with the given locality.
  pub allowed_destination: Option<Locality>,
  /// Attaches a cache to this publisher.
  ///
  /// The cache serves history and retransmission requests coming from subscribers.
  pub cache: Option<CacheConfig>,
  /// Allows matching subscribers to detect lost samples and optionally ask for
  /// retransmission.
  ///
  /// Retransmission can only be achieved if {@link PublisherOptions.cache} is enabled.
  pub sample_miss_detection: Option<MissDetectionConfig>,
  /// When `true`, allows this publisher to be detected by subscribers through liveliness,
  /// which lets them retrieve its local history.
  pub publisher_detection: Option<bool>,
}

/// Options for declaring a subscriber.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct SubscriberOptions {
  /// Restricts the matching publications received by this subscriber to the ones with
  /// the given locality.
  pub allowed_origin: Option<Locality>,
  /// Queries for historical data when the subscriber is declared.
  ///
  /// History can only be retransmitted by publishers that enable
  /// {@link PublisherOptions.cache}.
  pub history: Option<HistoryConfig>,
  /// Asks for retransmission of detected lost samples.
  ///
  /// Retransmission can only be achieved by publishers that enable both
  /// {@link PublisherOptions.cache} and {@link PublisherOptions.sampleMissDetection}.
  pub recovery: Option<napi::Either<PeriodicQueriesRecovery, HeartbeatRecovery>>,
  /// When `true`, allows this subscriber to be detected through liveliness.
  pub subscriber_detection: Option<bool>,
  /// A key expression appended to the liveliness token key expression.
  ///
  /// It can be used to convey metadata.
  pub subscriber_detection_metadata: Option<String>,
  /// Timeout, in milliseconds, for the queries issued for history and retransmission.
  ///
  /// Defaults to `10000`.
  pub query_timeout_ms: Option<f64>,
  /// Capacity of the channel buffering the received samples.
  ///
  /// Defaults to Zenoh's own reception channel size.
  pub channel_capacity: Option<u32>,
}

/// Options for a session put.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct PutOptions {
  /// Encoding of the payload.
  pub encoding: Option<String>,
  /// Congestion control to apply when routing the data.
  pub congestion_control: Option<CongestionControl>,
  /// Priority to apply when routing the data.
  pub priority: Option<Priority>,
  /// When `true`, the message is not batched.
  ///
  /// This usually has a positive impact on latency but a negative impact on throughput.
  pub express: Option<bool>,
  /// Reliability to apply when routing the data.
  ///
  /// Note: reliability does not trigger any data retransmission on the wire. It is a
  /// marker that may be used to select the best link available (e.g. TCP for reliable
  /// data and UDP for best effort data).
  pub reliability: Option<Reliability>,
  /// Restricts the matching subscribers that receive the published data to the ones
  /// with the given locality.
  pub allowed_destination: Option<Locality>,
  /// Arbitrary user-defined data sent alongside the payload.
  pub attachment: Option<BytesBuffer>,
}

/// Options for a session delete.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct DeleteOptions {
  /// Congestion control to apply when routing the data.
  pub congestion_control: Option<CongestionControl>,
  /// Priority to apply when routing the data.
  pub priority: Option<Priority>,
  /// When `true`, the message is not batched.
  ///
  /// This usually has a positive impact on latency but a negative impact on throughput.
  pub express: Option<bool>,
  /// Reliability to apply when routing the data.
  ///
  /// Note: reliability does not trigger any data retransmission on the wire. It is a
  /// marker that may be used to select the best link available (e.g. TCP for reliable
  /// data and UDP for best effort data).
  pub reliability: Option<Reliability>,
  /// Restricts the matching subscribers that receive the delete to the ones with the
  /// given locality.
  pub allowed_destination: Option<Locality>,
  /// Arbitrary user-defined data sent alongside the delete.
  pub attachment: Option<BytesBuffer>,
}

/// Configures the query for historical data performed by a subscriber on declaration.
#[napi(object, object_to_js = false)]
pub struct HistoryConfig {
  /// When `true`, detects late joiner publishers and queries their historical data.
  ///
  /// Late joiner detection only works with publishers that enable
  /// {@link PublisherOptions.publisherDetection}, and history can only be retransmitted
  /// by publishers that enable {@link PublisherOptions.cache}.
  pub detect_late_publishers: Option<bool>,
  /// How many samples to query for each resource.
  pub max_samples: Option<u32>,
  /// Maximum age, in seconds, of the samples to query.
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

/// Discriminant selecting {@link `PeriodicQueriesRecovery`}.
#[napi(string_enum)]
pub enum PeriodicQueriesMode {
  PeriodicQueries,
}

/// Discriminant selecting {@link `HeartbeatRecovery`}.
#[napi(string_enum)]
pub enum HeartbeatMode {
  Heartbeat,
}

/// Recovers missed samples by periodically querying for not yet received ones.
///
/// This allows retrieving the last sample(s) if they were lost, so it is useful for
/// sporadic publications but useless for periodic publications with a period smaller
/// than or equal to {@link PeriodicQueriesRecovery.periodMs}.
///
/// Retransmission can only be achieved by publishers that enable both
/// {@link PublisherOptions.cache} and {@link PublisherOptions.sampleMissDetection}.
#[napi(object, object_to_js = false)]
pub struct PeriodicQueriesRecovery {
  pub mode: PeriodicQueriesMode,
  /// Period of the queries, in milliseconds.
  pub period_ms: u32,
}

/// Recovers missed samples by subscribing to publisher heartbeats.
///
/// This allows receiving the last published sample's sequence number and checking for
/// misses. It must be paired with publishers that enable {@link PublisherOptions.cache}
/// and {@link PublisherOptions.sampleMissDetection} with a
/// {@link MissDetectionConfig.heartbeat}.
#[napi(object, object_to_js = false)]
pub struct HeartbeatRecovery {
  pub mode: HeartbeatMode,
}

impl From<PeriodicQueriesRecovery> for zenoh_ext::RecoveryConfig {
  fn from(value: PeriodicQueriesRecovery) -> Self {
    zenoh_ext::RecoveryConfig::<false>::default()
      .periodic_queries(Duration::from_millis(u64::from(value.period_ms)))
  }
}

impl From<HeartbeatRecovery> for zenoh_ext::RecoveryConfig {
  fn from(_value: HeartbeatRecovery) -> Self {
    zenoh_ext::RecoveryConfig::<false>::default().heartbeat()
  }
}

/// Options for scouting.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct ScoutOptions {
  /// Capacity of the channel buffering the received hellos.
  ///
  /// Defaults to Zenoh's own reception channel size.
  pub channel_capacity: Option<u32>,
}

/// Options for declaring a listener of matching status changes.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct MatchingListenerOptions {
  /// Capacity of the channel buffering the received matching statuses.
  ///
  /// Defaults to Zenoh's own reception channel size.
  pub channel_capacity: Option<u32>,
}

/// Options for declaring a listener of missed samples.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct SampleMissListenerOptions {
  /// Capacity of the channel buffering the received misses.
  ///
  /// Defaults to Zenoh's own reception channel size.
  pub channel_capacity: Option<u32>,
}

/// Options for declaring a listener of transport events.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct TransportEventsListenerOptions {
  /// When `true`, emits events for the existing transports before the live events.
  pub history: Option<bool>,
  /// Capacity of the channel buffering the received transport events.
  ///
  /// Defaults to Zenoh's own reception channel size.
  pub channel_capacity: Option<u32>,
}

/// Options for declaring a listener of link events.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct LinkEventsListenerOptions {
  /// When `true`, emits events for the existing links before the live events.
  pub history: Option<bool>,
  /// Capacity of the channel buffering the received link events.
  ///
  /// Defaults to Zenoh's own reception channel size.
  pub channel_capacity: Option<u32>,
}

/// Options for declaring a liveliness subscriber.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct LivelinessSubscriberOptions {
  /// When `true`, queries the network for the currently live tokens upon declaring the
  /// subscriber.
  ///
  /// When `false`, no such query is made, though currently live tokens may still be
  /// delivered to the subscriber.
  pub history: Option<bool>,
  /// Capacity of the channel buffering the received samples.
  ///
  /// Defaults to Zenoh's own reception channel size.
  pub channel_capacity: Option<u32>,
}

/// Options for a liveliness query.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct LivelinessGetOptions {
  /// Query timeout, in milliseconds.
  ///
  /// Defaults to the session's `queries_default_timeout` configuration.
  pub timeout: Option<f64>,
  // pub cancellation_token: Option<Instance<CancellationToken>>,
  /// Capacity of the channel buffering the received replies.
  ///
  /// Defaults to Zenoh's own reception channel size.
  pub channel_capacity: Option<u32>,
}

/// Configures the heartbeat published for sample miss detection.
#[napi(object, object_to_js = false)]
pub struct HeartbeatConfig {
  /// Period, in milliseconds, at which the last published sample's sequence number is
  /// sent.
  pub period_ms: u32,
  /// When `true`, the sequence number is sent each period only if it changed since the
  /// last one, and it is sent with a blocking congestion control.
  pub sporadic: Option<bool>,
}

/// Configuration for sample miss detection.
///
/// Enabling {@link PublisherOptions.sampleMissDetection} allows subscribers to detect
/// missed samples through a sample miss listener and to recover them through
/// {@link SubscriberOptions.recovery}.
#[napi(object, object_to_js = false)]
pub struct MissDetectionConfig {
  /// Allows last sample miss detection by periodically publishing the last sample's
  /// sequence number.
  ///
  /// Subscribers can recover the last sample with {@link `HeartbeatRecovery`}.
  pub heartbeat: Option<HeartbeatConfig>,
}

impl From<MissDetectionConfig> for zenoh_ext::MissDetectionConfig {
  fn from(value: MissDetectionConfig) -> Self {
    let mut config = zenoh_ext::MissDetectionConfig::default();
    if let Some(heartbeat) = value.heartbeat {
      let period = Duration::from_millis(u64::from(heartbeat.period_ms));
      config = if heartbeat.sporadic == Some(true) {
        config.sporadic_heartbeat(period)
      } else {
        config.heartbeat(period)
      };
    }
    config
  }
}

/// Configures a publisher's cache, used to serve history and retransmissions.
#[napi(object, object_to_js = false)]
pub struct CacheConfig {
  /// How many samples to keep for each resource.
  ///
  /// Defaults to `1`.
  pub max_samples: Option<u32>,
  /// `QoS` to apply to the replies served from the cache.
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

/// `QoS` applied to the replies served from a cache.
#[napi(object)]
pub struct RepliesConfig {
  /// Priority to apply when routing the replies.
  pub priority: Option<Priority>,
  /// Congestion control to apply when routing the replies.
  pub congestion_control: Option<CongestionControl>,
  /// When `true`, the replies are not batched.
  ///
  /// This usually has a positive impact on latency but a negative impact on throughput.
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

/// Options for replying to a query with a payload.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct ReplyOptions {
  /// Encoding of the reply payload.
  pub encoding: Option<String>,
  /// When `true`, the reply is not batched.
  ///
  /// This usually has a positive impact on latency but a negative impact on throughput.
  pub express: Option<bool>,
  /// Arbitrary user-defined data sent alongside the reply payload.
  pub attachment: Option<BytesBuffer>,
}

/// Options for replying to a query with an error.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct ReplyErrOptions {
  /// Encoding of the error payload.
  pub encoding: Option<String>,
}

/// Options for replying to a query with a delete.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct ReplyDelOptions {
  /// When `true`, the reply is not batched.
  ///
  /// This usually has a positive impact on latency but a negative impact on throughput.
  pub express: Option<bool>,
  /// Arbitrary user-defined data sent alongside the reply.
  pub attachment: Option<BytesBuffer>,
}

/// Options for a query issued by a querier.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct QuerierGetOptions {
  /// Selector parameters of this query.
  pub parameters: Option<ParametersLike>,
  /// Payload sent along with the query.
  pub payload: Option<BytesBuffer>,
  /// Encoding of the query payload.
  pub encoding: Option<String>,
  /// Arbitrary user-defined data sent alongside the query.
  pub attachment: Option<BytesBuffer>,
  // pub cancellation_token: Option<Instance<CancellationToken>>,
  /// Capacity of the channel buffering the received replies.
  ///
  /// Defaults to Zenoh's own reception channel size.
  pub channel_capacity: Option<u32>,
}

/// Options for declaring a queryable.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct QueryableOptions {
  /// When `true`, the queryable promises to have all the data associated with its key
  /// expression, so queriers do not need to query other nodes for matching data.
  ///
  /// E.g. a queryable serving `foo/*` that is complete answers a query for `foo/bar` on
  /// its own, even if other queryables match `foo/bar`. But a complete queryable serving
  /// `foo/bar` does not cover the whole of `foo/*`, so a query for `foo/*` is still sent
  /// to other queryables as well.
  ///
  /// This applies to the default {@link `QueryTarget`} `BestMatching`. `All` forcibly
  /// requests every available queryable, and `AllComplete` requests only the complete
  /// ones.
  pub complete: Option<bool>,
  /// Restricts the matching queries received by this queryable to the ones with the given
  /// locality.
  pub allowed_origin: Option<Locality>,
  /// Capacity of the channel buffering the received queries.
  ///
  /// Defaults to Zenoh's own reception channel size.
  pub channel_capacity: Option<u32>,
}

/// Options for declaring a querier.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct QuerierOptions {
  /// Target(s) of the querier's queries.
  ///
  /// Selects whether a query just returns the data available in the network matching the
  /// key expression (`BestMatching`, the default) or whether it reaches all matching
  /// queryables (`All`, `AllComplete`).
  ///
  /// See also {@link QueryableOptions.complete}.
  pub target: Option<QueryTarget>,
  /// Consolidation mode of the querier's queries.
  ///
  /// Multiple replies to a query may arrive from the network; the consolidation mode
  /// defines the strategy for filtering and reordering them. `Auto` lets the
  /// implementation choose the best strategy.
  pub consolidation: Option<ConsolidationMode>,
  /// Congestion control to apply when routing the queries.
  pub congestion_control: Option<CongestionControl>,
  /// Priority to apply when routing the queries.
  pub priority: Option<Priority>,
  /// When `true`, the queries are not batched.
  ///
  /// This usually has a positive impact on latency but a negative impact on throughput.
  pub express: Option<bool>,
  /// Restricts the matching queryables that receive the queries to the ones with the
  /// given locality.
  pub allowed_destination: Option<Locality>,
  /// Query timeout, in milliseconds.
  ///
  /// Defaults to the session's `queries_default_timeout` configuration.
  pub timeout: Option<f64>,
  /// Whether this querier accepts replies whose key expression does not intersect its
  /// own.
  ///
  /// A queryable serving a glob-like key expression such as `foo/*` may reply to a query
  /// for `foo/bar` with the key expression `foo/baz`. By default such disjoint replies
  /// are rejected on the sending side; `Any` accepts them.
  pub accept_replies: Option<ReplyKeyExpr>,
}

/// Options for a session query.
#[derive(Default)]
#[napi(object, object_to_js = false)]
pub struct GetOptions {
  /// Selector parameters of this query.
  ///
  /// Replaces any parameters already carried by the selector argument.
  pub parameters: Option<ParametersLike>,
  /// Target(s) of the query.
  ///
  /// Selects whether the query just returns the data available in the network matching
  /// the key expression (`BestMatching`, the default) or whether it reaches all matching
  /// queryables (`All`, `AllComplete`).
  ///
  /// See also {@link QueryableOptions.complete}.
  pub target: Option<QueryTarget>,
  /// Consolidation mode of the query.
  ///
  /// Multiple replies to a query may arrive from the network; the consolidation mode
  /// defines the strategy for filtering and reordering them. `Auto` lets the
  /// implementation choose the best strategy.
  pub consolidation: Option<ConsolidationMode>,
  /// Congestion control to apply when routing the query.
  pub congestion_control: Option<CongestionControl>,
  /// Priority to apply when routing the query.
  pub priority: Option<Priority>,
  /// When `true`, the query is not batched.
  ///
  /// This usually has a positive impact on latency but a negative impact on throughput.
  pub express: Option<bool>,
  /// Restricts the matching queryables that receive the query to the ones with the given
  /// locality.
  pub allowed_destination: Option<Locality>,
  /// Query timeout, in milliseconds.
  ///
  /// Defaults to the session's `queries_default_timeout` configuration.
  pub timeout: Option<f64>,
  /// Payload sent along with the query.
  pub payload: Option<BytesBuffer>,
  /// Encoding of the query payload.
  pub encoding: Option<String>,
  /// Arbitrary user-defined data sent alongside the query.
  pub attachment: Option<BytesBuffer>,
  // pub cancellation_token: Option<ClassInstance<'env, CancellationToken>>,
  /// Capacity of the channel buffering the received replies.
  ///
  /// Defaults to Zenoh's own reception channel size.
  pub channel_capacity: Option<u32>,
}
