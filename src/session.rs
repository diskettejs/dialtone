use napi_derive::napi;
use zenoh::{Wait, config as zconfig, session as zsession};
use zenoh_ext::{AdvancedPublisherBuilderExt, AdvancedSubscriberBuilderExt};

use crate::{
  bytes::*, config::*, handlers::*, key_expr::*, liveliness::*, macros::*, options::*, pubsub::*,
  qos::Reliability, query::*, sample::SampleKind, time::*, utils::*,
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
    payload: BytesLike,
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
  ) -> napi::Result<Handler> {
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
      channel,
    } = options.unwrap_or_default();

    let selector = Selector::resolve(selector, parameters)?;
    let timeout = duration_ms(timeout)?;
    let handler = into_handler(channel);
    let session = self.inner.clone();

    let receiver = build!(
      session.get(selector).with(handler),
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

    Ok(receiver.into())
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

#[napi]
pub struct SessionInfo {
  inner: zsession::SessionInfo,
}

impl From<zsession::SessionInfo> for SessionInfo {
  fn from(value: zsession::SessionInfo) -> Self {
    Self { inner: value }
  }
}

#[napi]
impl SessionInfo {
  #[napi]
  pub async fn zid(&self) -> String {
    self.inner.zid().await.to_string()
  }

  #[napi]
  pub async fn routers_zid(&self) -> Vec<String> {
    self
      .inner
      .routers_zid()
      .await
      .map(|zid| zid.to_string())
      .collect()
  }

  #[napi]
  pub async fn peers_zid(&self) -> Vec<String> {
    self
      .inner
      .peers_zid()
      .await
      .map(|zid| zid.to_string())
      .collect()
  }

  #[napi]
  pub async fn transports(&self) -> Vec<Transport> {
    self.inner.transports().await.map(Transport::from).collect()
  }

  #[napi]
  pub async fn links(&self) -> Vec<Link> {
    self.inner.links().await.map(Link::from).collect()
  }

  #[napi]
  pub async fn transport_events_listener(
    &self,
    options: Option<TransportEventsListenerOptions>,
  ) -> napi::Result<TransportEventsListener> {
    let TransportEventsListenerOptions { history, channel } = options.unwrap_or_default();
    let handler = into_handler(channel);

    let listener = build!(
      self.inner.transport_events_listener().with(handler),
      history,
    )
    .await
    .map_napi_err()?;

    Ok(listener.into())
  }

  #[napi]
  pub async fn link_events_listener(
    &self,
    options: Option<LinkEventsListenerOptions>,
  ) -> napi::Result<LinkEventsListener> {
    let LinkEventsListenerOptions {
      history,
      transport,
      channel,
    } = options.unwrap_or_default();
    let handler = into_handler(channel);

    let listener = build!(
      self.inner.link_events_listener().with(handler),
      history,
      transport,
    )
    .await
    .map_napi_err()?;

    Ok(listener.into())
  }
}

wrapper!(zsession::Transport: Clone);

#[napi]
impl Transport {
  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.inner.zid().to_string()
  }

  #[napi(getter)]
  pub fn whatami(&self) -> WhatAmI {
    self.inner.whatami().into()
  }

  #[napi(getter)]
  pub fn is_qos(&self) -> bool {
    self.inner.is_qos()
  }

  #[napi(getter)]
  pub fn is_multicast(&self) -> bool {
    self.inner.is_multicast()
  }
}

wrapper!(zsession::TransportEvent);

#[napi]
impl TransportEvent {
  #[napi(getter)]
  pub fn kind(&self) -> SampleKind {
    self.inner.kind().into()
  }

  #[napi(getter)]
  pub fn transport(&self) -> Transport {
    self.inner.transport().clone().into()
  }
}

option_wrapper!(
  zsession::TransportEventsListener<HandlerImpl<zsession::TransportEvent>>,
  "Undeclared transport events listener"
);

#[napi]
impl TransportEventsListener {
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    Wait::wait(self.take()?.undeclare()).map_napi_err()
  }
}

recv_handler!(TransportEventsListener => TransportEvent);

#[napi(object)]
pub struct LinkPriorities {
  pub min: u8,
  pub max: u8,
}

wrapper!(zsession::Link);

#[napi]
impl Link {
  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.inner.zid().to_string()
  }

  #[napi(getter)]
  pub fn src(&self) -> Locator {
    self.inner.src().clone().into()
  }

  #[napi(getter)]
  pub fn dst(&self) -> Locator {
    self.inner.dst().clone().into()
  }

  #[napi(getter)]
  pub fn group(&self) -> Option<Locator> {
    self.inner.group().cloned().map(Locator::from)
  }

  #[napi(getter)]
  pub fn mtu(&self) -> u16 {
    self.inner.mtu()
  }

  #[napi(getter)]
  pub fn is_streamed(&self) -> bool {
    self.inner.is_streamed()
  }

  #[napi(getter)]
  pub fn interfaces(&self) -> Vec<String> {
    self.inner.interfaces().to_vec()
  }

  #[napi(getter)]
  pub fn auth_identifier(&self) -> Option<String> {
    self.inner.auth_identifier().map(|s| s.to_string())
  }

  #[napi(getter)]
  pub fn priorities(&self) -> Option<LinkPriorities> {
    self
      .inner
      .priorities()
      .map(|(min, max)| LinkPriorities { min, max })
  }

  #[napi(getter)]
  pub fn reliability(&self) -> Option<Reliability> {
    self.inner.reliability().map(Into::into)
  }
}

wrapper!(zsession::LinkEvent);

#[napi]
impl LinkEvent {
  #[napi(getter)]
  pub fn kind(&self) -> SampleKind {
    self.inner.kind().into()
  }

  #[napi(getter)]
  pub fn link(&self) -> Link {
    self.inner.link().clone().into()
  }
}

option_wrapper!(
  zsession::LinkEventsListener<HandlerImpl<zsession::LinkEvent>>,
  "Undeclared link events listener"
);

#[napi]
impl LinkEventsListener {
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    Wait::wait(self.take()?.undeclare()).map_napi_err()
  }
}

recv_handler!(LinkEventsListener => LinkEvent);

wrapper!(zconfig::Locator);

#[napi]
impl Locator {
  #[napi(constructor)]
  pub fn new(protocol: String, address: String, metadata: String) -> napi::Result<Self> {
    let inner = zconfig::Locator::new(protocol, address, metadata).map_napi_err()?;
    Ok(inner.into())
  }

  #[napi(getter)]
  pub fn protocol(&self) -> String {
    self.inner.protocol().as_str().to_string()
  }

  #[napi(getter)]
  pub fn address(&self) -> String {
    self.inner.address().as_str().to_string()
  }

  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.inner.as_str().to_string()
  }

  #[napi]
  pub fn metadata(&self) -> Metadata {
    self.inner.to_endpoint().into()
  }

  #[napi]
  pub fn to_endpoint(&self) -> EndPoint {
    self.inner.to_endpoint().into()
  }
}

wrapper!(zconfig::EndPoint as Metadata);

#[napi]
impl Metadata {
  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.inner.metadata().as_str().to_string()
  }

  #[napi]
  pub fn is_empty(&self) -> bool {
    self.inner.metadata().is_empty()
  }

  #[napi]
  pub fn get(&self, key: String) -> Option<String> {
    self
      .inner
      .metadata()
      .get(&key)
      .map(|value| value.to_string())
  }

  #[napi]
  pub fn values(&self, key: String) -> Vec<String> {
    self
      .inner
      .metadata()
      .values(&key)
      .map(|value| value.to_string())
      .collect()
  }
}

wrapper!(zconfig::EndPoint);

#[napi(object)]
pub struct EndPointParts {
  pub protocol: String,
  pub address: String,
  pub metadata: String,
  pub config: String,
}

#[napi]
impl EndPoint {
  #[napi(constructor)]
  pub fn new(
    protocol: String,
    address: String,
    metadata: String,
    config: String,
  ) -> napi::Result<Self> {
    let inner = zconfig::EndPoint::new(protocol, address, metadata, config).map_napi_err()?;

    Ok(inner.into())
  }

  #[napi(getter)]
  pub fn protocol(&self) -> String {
    self.inner.protocol().as_str().to_string()
  }

  #[napi(getter)]
  pub fn address(&self) -> String {
    self.inner.address().as_str().to_string()
  }

  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.inner.as_str().to_string()
  }

  #[napi]
  pub fn metadata(&self) -> Metadata {
    self.inner.clone().into()
  }

  #[napi]
  pub fn config(&self) -> String {
    self.inner.config().as_str().to_string()
  }

  #[napi]
  pub fn split(&self) -> EndPointParts {
    let (protocol, address, metadata, config) = self.inner.split();
    EndPointParts {
      protocol: protocol.as_str().to_string(),
      address: address.as_str().to_string(),
      metadata: metadata.as_str().to_string(),
      config: config.as_str().to_string(),
    }
  }

  #[napi]
  pub fn to_locator(&self) -> Locator {
    self.inner.to_locator().into()
  }
}
