use napi_derive::napi;
use zenoh::{Wait, handlers::IntoHandler};
use zenoh_ext::{AdvancedPublisherBuilderExt, AdvancedSubscriberBuilderExt};

use crate::{
  bytes::*, channels::*, config::*, error::*, handlers::into_handler, info::*, key_expr::*,
  liveliness::*, macros::*, options::*, publisher::*, querier::*, queryable::*, selector::*,
  subscriber::*, time::*,
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
  pub async fn put(
    &self,
    key_expr: KeyExprArg<'_>,
    payload: Payload,
    options: Option<PutOptions>,
  ) -> napi::Result<()> {
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
    let session = self.inner.clone();

    build!(
      session.put(expr, payload),
      encoding,
      congestion_control,
      priority,
      express,
      reliability,
      allowed_destination,
      timestamp,
      attachment,
      source_info,
    )
    .await
    .map_napi_err()
  }

  #[napi]
  pub async fn get(
    &self,
    selector: SelectorArg<'_>,
    options: Option<GetOptions>,
  ) -> napi::Result<()> {
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
    } = options.unwrap_or_default();

    // NOTE: temp hardcoded because of ongoing channel handlers rework
    let (cb, receiver) = FifoChannel::new(256).into_handler();

    let selector = Selector::resolve(selector, parameters)?;
    let timeout = duration_ms(timeout)?;
    let session = self.inner.clone();

    build!(
      session.get(selector).with(cb),
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
    )
    .await
    .map_napi_err()?;

    // Ok(receiver.into())
    todo!("migration to new channel handlers")
  }

  #[napi]
  pub async fn delete(
    &self,
    key_expr: KeyExprArg<'_>,
    options: Option<DeleteOptions>,
  ) -> napi::Result<()> {
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
    let session = self.inner.clone();

    build!(
      session.delete(expr),
      congestion_control,
      priority,
      express,
      reliability,
      allowed_destination,
      timestamp,
      attachment,
      source_info,
    )
    .await
    .map_napi_err()
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
    let timeout = duration_ms(timeout)?;

    let zquerier = build!(
      self.inner.declare_querier(expr),
      target,
      consolidation,
      congestion_control,
      priority,
      express,
      allowed_destination,
      timeout,
      accept_replies,
    )
    .await
    .map_napi_err()?;

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
      channel,
    } = options.unwrap_or_default();
    let expr = KeyExpr::try_from(key_expr)?;
    let handler = into_handler(channel);

    let queryable = build!(
      self.inner.declare_queryable(expr).with(handler),
      allowed_origin,
      complete,
    )
    .await
    .map_napi_err()?;

    Ok(queryable.into())
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
      channel,
    } = options.unwrap_or_default();
    let query_timeout = duration_ms(query_timeout_ms)?;
    let handler = into_handler(channel);
    let session = self.inner.clone();

    let base = session
      .declare_subscriber(key_expr)
      .advanced()
      .with(handler);

    let mut builder = build!(
      base,
      allowed_origin,
      history,
      recovery,
      query_timeout,
      subscriber_detection_metadata,
    );

    if subscriber_detection == Some(true) {
      builder = builder.subscriber_detection();
    }

    let subscriber = builder.await.map_napi_err()?;

    Ok(subscriber.into())
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

    let mut builder = build!(
      self.inner.declare_publisher(expr).advanced(),
      encoding,
      congestion_control,
      priority,
      express,
      reliability,
      allowed_destination,
      cache,
      publisher_detection_metadata,
      sample_miss_detection,
    );

    if publisher_detection == Some(true) {
      builder = builder.publisher_detection();
    }

    let publisher = builder.await.map_napi_err()?;

    Ok(publisher.into())
  }
}
