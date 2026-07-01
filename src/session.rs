use std::time::Duration;

use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{
  cancellation as zcancellation, handlers::IntoHandler, query as zquery, sample as zsample,
  time as ztime,
};
use zenoh_ext::{AdvancedPublisherBuilderExt, AdvancedSubscriberBuilderExt};

use crate::{
  bytes::*, channels::*, config::*, error::*, handlers::*, info::*, key_expr::*, liveliness::*,
  macros::wrapper, options::*, publisher::*, querier::*, queryable::*, selector::*, subscriber::*,
  time::*,
};

wrapper!(zenoh::Session);

#[napi]
impl Session {
  #[napi(factory)]
  pub async fn open(config: &Config) -> napi::Result<Self> {
    let cfg = zenoh::config::Config::from(config);
    let session = zenoh::open(cfg).await.map_napi_err()?;
    Ok(session.into())
  }

  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.inner.zid().to_string()
  }

  #[napi(getter)]
  pub fn id(&self) -> EntityGlobalId {
    self.inner.id().into()
  }

  #[napi(getter)]
  pub fn is_closed(&self) -> bool {
    self.inner.is_closed()
  }

  #[napi]
  pub fn new_timestamp(&self) -> Timestamp {
    self.inner.new_timestamp().into()
  }

  #[napi]
  pub fn info(&self) -> SessionInfo {
    self.inner.info().into()
  }

  #[napi]
  pub fn config(&self) -> SessionConfig {
    self.inner.config().into()
  }

  #[napi]
  pub async fn close(&self) -> napi::Result<()> {
    self.inner.close().await.map_napi_err()
  }

  #[napi]
  pub fn put<'env>(
    &self,
    env: &'env Env,
    key_expr: KeyExprArg,
    payload: PayloadArg,
    options: Option<PutOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, ()>> {
    let expr = KeyExpr::try_from(key_expr)?;
    let payload = payload.into_zbytes();
    let PutOptions {
      encoding,
      congestion_control,
      priority,
      express,
      reliability,
      allowed_destination,
      timestamp,
      attachment,
      source_info,
    } = options.unwrap_or_default();

    let timestamp = timestamp.map(|ts| ztime::Timestamp::from(ts.as_ref()));
    let attachment = attachment.map(|attachment| attachment.into_zbytes());
    let source_info = source_info.map(|source_info| zsample::SourceInfo::from(&*source_info));
    let session = self.inner.clone();

    env.spawn_future(async move {
      let mut builder = session.put(expr, payload);

      if let Some(encoding) = encoding {
        builder = builder.encoding(encoding);
      }

      if let Some(congestion_control) = congestion_control {
        builder = builder.congestion_control(congestion_control.into());
      }

      if let Some(priority) = priority {
        builder = builder.priority(priority.into());
      }

      if let Some(express) = express {
        builder = builder.express(express);
      }

      if let Some(reliability) = reliability {
        builder = builder.reliability(reliability.into());
      }

      if let Some(allowed_destination) = allowed_destination {
        builder = builder.allowed_destination(allowed_destination.into());
      }

      if let Some(timestamp) = timestamp {
        builder = builder.timestamp(timestamp);
      }

      if let Some(attachment) = attachment {
        builder = builder.attachment(attachment);
      }

      if let Some(source_info) = source_info {
        builder = builder.source_info(source_info);
      }

      builder.await.map_napi_err()
    })
  }

  #[napi]
  pub fn get<'env>(
    &self,
    env: &'env Env,
    selector: SelectorArg<'_>,
    options: Option<GetOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, Replies>> {
    let GetOptions {
      parameters,
      target,
      consolidation,
      congestion_control,
      priority,
      express,
      allowed_destination,
      timeout,
      payload,
      encoding,
      attachment,
      source_info,
      cancellation_token,
      capacity,
    } = options.unwrap_or_default();
    let (cb, receiver) = FifoChannel::with_capacity(capacity).into_handler();

    let mut selector = zquery::Selector::from(Selector::try_from(selector)?);
    if let Some(parameters) = parameters {
      let key_expr = selector.key_expr().clone().into_owned();
      selector = zquery::Selector::owned(key_expr, zquery::Parameters::from(&*parameters));
    }

    let timeout = timeout
      .map(|ms| Duration::try_from_secs_f64(ms / 1000.0).map_napi_err())
      .transpose()?;
    let payload = payload.map(|payload| payload.into_zbytes());
    let attachment = attachment.map(|attachment| attachment.into_zbytes());
    let source_info = source_info.map(|source_info| zsample::SourceInfo::from(&*source_info));
    let cancellation_token =
      cancellation_token.map(|ct| zcancellation::CancellationToken::from(&*ct));
    let session = self.inner.clone();

    env.spawn_future(async move {
      let mut builder = session.get(selector).with(cb);

      if let Some(target) = target {
        builder = builder.target(target.into());
      }

      if let Some(consolidation) = consolidation {
        builder = builder.consolidation(zquery::ConsolidationMode::from(consolidation));
      }

      if let Some(congestion_control) = congestion_control {
        builder = builder.congestion_control(congestion_control.into());
      }

      if let Some(priority) = priority {
        builder = builder.priority(priority.into());
      }

      if let Some(express) = express {
        builder = builder.express(express);
      }

      if let Some(allowed_destination) = allowed_destination {
        builder = builder.allowed_destination(allowed_destination.into());
      }

      if let Some(timeout) = timeout {
        builder = builder.timeout(timeout);
      }

      if let Some(payload) = payload {
        builder = builder.payload(payload);
      }

      if let Some(encoding) = encoding {
        builder = builder.encoding(encoding);
      }

      if let Some(attachment) = attachment {
        builder = builder.attachment(attachment);
      }

      if let Some(source_info) = source_info {
        builder = builder.source_info(source_info);
      }

      if let Some(cancellation_token) = cancellation_token {
        builder = builder.cancellation_token(cancellation_token);
      }

      builder.await.map_napi_err()?;
      Ok(receiver.into())
    })
  }

  #[napi]
  pub fn delete<'env>(
    &self,
    env: &'env Env,
    key_expr: KeyExprArg,
    options: Option<DeleteOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, ()>> {
    let expr = KeyExpr::try_from(key_expr)?;
    let DeleteOptions {
      congestion_control,
      priority,
      express,
      reliability,
      allowed_destination,
      timestamp,
      attachment,
      source_info,
    } = options.unwrap_or_default();

    let timestamp = timestamp.map(|ts| ztime::Timestamp::from(ts.as_ref()));
    let attachment = attachment.map(|attachment| attachment.into_zbytes());
    let source_info = source_info.map(|source_info| zsample::SourceInfo::from(&*source_info));
    let session = self.inner.clone();

    env.spawn_future(async move {
      let mut builder = session.delete(expr);

      if let Some(congestion_control) = congestion_control {
        builder = builder.congestion_control(congestion_control.into());
      }

      if let Some(priority) = priority {
        builder = builder.priority(priority.into());
      }

      if let Some(express) = express {
        builder = builder.express(express);
      }

      if let Some(reliability) = reliability {
        builder = builder.reliability(reliability.into());
      }

      if let Some(allowed_destination) = allowed_destination {
        builder = builder.allowed_destination(allowed_destination.into());
      }

      if let Some(timestamp) = timestamp {
        builder = builder.timestamp(timestamp);
      }

      if let Some(attachment) = attachment {
        builder = builder.attachment(attachment);
      }

      if let Some(source_info) = source_info {
        builder = builder.source_info(source_info);
      }

      builder.await.map_napi_err()
    })
  }

  #[napi]
  pub fn liveliness(&self) -> Liveliness {
    self.inner.clone().into()
  }

  #[napi]
  pub async fn declare_keyexpr(&self, key_expr: KeyExprArg<'_>) -> napi::Result<KeyExpr> {
    let expr = KeyExpr::try_from(key_expr)?;
    let keyexpr = self.inner.declare_keyexpr(expr).await.map_napi_err()?;

    Ok(keyexpr.into())
  }

  #[napi]
  pub async fn declare_querier(
    &self,
    key_expr: KeyExprArg<'_>,
    options: Option<QuerierOptions>,
  ) -> napi::Result<Querier> {
    let QuerierOptions {
      target,
      consolidation,
      congestion_control,
      priority,
      express,
      allowed_destination,
      timeout,
      accept_replies,
    } = options.unwrap_or_default();
    let expr = KeyExpr::try_from(key_expr)?;
    let mut builder = self.inner.declare_querier(expr);

    if let Some(target) = target {
      builder = builder.target(target.into());
    }

    if let Some(consolidation) = consolidation {
      builder = builder.consolidation(zquery::ConsolidationMode::from(consolidation));
    }

    if let Some(congestion_control) = congestion_control {
      builder = builder.congestion_control(congestion_control.into());
    }

    if let Some(priority) = priority {
      builder = builder.priority(priority.into());
    }

    if let Some(express) = express {
      builder = builder.express(express);
    }

    if let Some(allowed_destination) = allowed_destination {
      builder = builder.allowed_destination(allowed_destination.into());
    }

    if let Some(timeout) = timeout {
      builder = builder.timeout(Duration::try_from_secs_f64(timeout / 1000.0).map_napi_err()?);
    }

    if let Some(accept_replies) = accept_replies {
      builder = builder.accept_replies(accept_replies.into());
    }

    let zquerier = builder.await.map_napi_err()?;

    Ok(zquerier.into())
  }

  #[napi]
  pub async fn declare_queryable(
    &self,
    key_expr: KeyExprArg<'_>,
    options: Option<QueryableOptions>,
  ) -> napi::Result<Queryable> {
    let QueryableOptions {
      allowed_origin,
      complete,
      capacity,
    } = options.unwrap_or_default();
    let expr = KeyExpr::try_from(key_expr)?;
    let (cb, receiver) = FifoChannel::with_capacity(capacity).into_handler();
    let mut builder = self.inner.declare_queryable(expr).with((cb, ()));

    if let Some(origin) = allowed_origin {
      builder = builder.allowed_origin(origin.into());
    };

    if let Some(complete) = complete {
      builder = builder.complete(complete);
    }

    let queryable = builder.await.map_napi_err()?;

    Ok(Queryable::new(queryable, receiver))
  }

  #[napi]
  pub async fn declare_subscriber(
    &self,
    key_expr: KeyExprArg<'_>,
    options: Option<SubscriberOptions>,
  ) -> napi::Result<Subscriber> {
    let key_expr = KeyExpr::try_from(key_expr)?;
    let SubscriberOptions {
      allowed_origin,
      history,
      query_timeout_ms,
      recovery,
      subscriber_detection,
      subscriber_detection_metadata,
      capacity,
    } = options.unwrap_or_default();
    let (cb, receiver) = FifoChannel::with_capacity(capacity).into_handler();
    let mut builder = self
      .inner
      .declare_subscriber(key_expr)
      .advanced()
      .with((cb, ()));

    if let Some(allowed_origin) = allowed_origin {
      builder = builder.allowed_origin(allowed_origin.into());
    }

    if let Some(history) = history {
      builder = builder.history(history.into());
    }

    if let Some(recovery) = recovery {
      let recovery: zenoh_ext::RecoveryConfig = match recovery {
        Either::A(periodic) => periodic.into(),
        Either::B(heartbeat) => heartbeat.into(),
      };
      builder = builder.recovery(recovery);
    }

    if let Some(query_timeout_ms) = query_timeout_ms {
      let timeout = Duration::try_from_secs_f64(query_timeout_ms / 1000.0).map_napi_err()?;
      builder = builder.query_timeout(timeout);
    }

    if subscriber_detection == Some(true) {
      builder = builder.subscriber_detection();
    }

    if let Some(subscriber_detection_metadata) = subscriber_detection_metadata {
      builder = builder.subscriber_detection_metadata(subscriber_detection_metadata);
    }

    let subscriber = builder.await.map_napi_err()?;

    Ok(Subscriber::new(subscriber, receiver))
  }

  #[napi]
  pub async fn declare_publisher(
    &self,
    key_expr: KeyExprArg<'_>,
    options: Option<PublisherOptions>,
  ) -> napi::Result<Publisher> {
    let expr = KeyExpr::try_from(key_expr)?;
    let PublisherOptions {
      cache,
      encoding,
      congestion_control,
      express,
      allowed_destination,
      priority,
      publisher_detection,
      publisher_detection_metadata,
      reliability,
      sample_miss_detection,
    } = options.unwrap_or_default();
    let mut builder = self.inner.declare_publisher(expr).advanced();

    if let Some(encoding) = encoding {
      builder = builder.encoding(encoding);
    }

    if let Some(congestion_control) = congestion_control {
      builder = builder.congestion_control(congestion_control.into());
    }

    if let Some(priority) = priority {
      builder = builder.priority(priority.into());
    }

    if let Some(express) = express {
      builder = builder.express(express);
    }

    if let Some(reliability) = reliability {
      builder = builder.reliability(reliability.into());
    }

    if let Some(allowed_destination) = allowed_destination {
      builder = builder.allowed_destination(allowed_destination.into());
    }

    if let Some(cache) = cache {
      builder = builder.cache(cache.into());
    }

    if publisher_detection == Some(true) {
      builder = builder.publisher_detection();
    }

    if let Some(publisher_detection_metadata) = publisher_detection_metadata {
      builder = builder.publisher_detection_metadata(publisher_detection_metadata);
    }

    if let Some(sample_miss_detection) = sample_miss_detection {
      builder = builder.sample_miss_detection(sample_miss_detection.into());
    }

    let publisher = builder.await.map_napi_err()?;

    Ok(publisher.into())
  }
}
