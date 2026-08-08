use derive_more::{From, Into};
use napi_derive::napi;
use zenoh as z;
use zenoh_ext::{AdvancedPublisherBuilderExt, AdvancedSubscriberBuilderExt};

use crate::{
  bytes::*, config::*, handlers::*, key_expr::*, liveliness::*, options::*, pubsub::*,
  qos::Reliability, query::*, sample::SampleKind, time::*, utils::*,
};

#[derive(From, Into)]
#[napi]
pub struct Session(z::Session);

#[napi]
impl Session {
  #[napi(factory)]
  pub async fn open(config: &Config) -> napi::Result<Self> {
    let cfg = config.as_ref().clone();
    let session = z::open(cfg).await.map_napi_err()?;
    Ok(session.into())
  }

  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.0.zid().to_string()
  }

  #[napi(getter)]
  pub fn id(&self) -> EntityGlobalId {
    self.0.id().into()
  }

  #[napi(getter)]
  pub fn is_closed(&self) -> bool {
    self.0.is_closed()
  }

  #[napi]
  pub fn new_timestamp(&self) -> Timestamp {
    self.0.new_timestamp().into()
  }

  #[napi]
  pub fn info(&self) -> SessionInfo {
    self.0.info().into()
  }

  #[napi]
  pub fn config(&self) -> SessionConfig {
    self.0.config().into()
  }

  #[napi]
  pub async fn close(&self) -> napi::Result<()> {
    self.0.close().await.map_napi_err()
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
    let session = self.0.clone();

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
    let session = self.0.clone();

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
    let session = self.0.clone();

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
    self.0.clone().into()
  }

  #[napi]
  pub async fn declare_keyexpr(&self, key_expr: KeyExprArg<'_>) -> napi::Result<KeyExpr> {
    let expr = KeyExpr::try_from(key_expr)?;
    let keyexpr = self.0.declare_keyexpr(expr).await.map_napi_err()?;

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
      self.0.declare_querier(expr),
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
      self.0.declare_queryable(expr).with(handler),
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
    let session = self.0.clone();

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
      self.0.declare_publisher(expr).advanced(),
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

#[derive(From)]
#[napi]
pub struct SessionInfo(z::session::SessionInfo);

#[napi]
impl SessionInfo {
  #[napi]
  pub async fn zid(&self) -> String {
    self.0.zid().await.to_string()
  }

  #[napi]
  pub async fn routers_zid(&self) -> Vec<String> {
    self
      .0
      .routers_zid()
      .await
      .map(|zid| zid.to_string())
      .collect()
  }

  #[napi]
  pub async fn peers_zid(&self) -> Vec<String> {
    self
      .0
      .peers_zid()
      .await
      .map(|zid| zid.to_string())
      .collect()
  }

  #[napi]
  pub async fn transports(&self) -> Vec<Transport> {
    self.0.transports().await.map(Transport::from).collect()
  }

  #[napi]
  pub async fn links(&self) -> Vec<Link> {
    self.0.links().await.map(Link::from).collect()
  }

  #[napi]
  pub async fn transport_events_listener(
    &self,
    options: Option<TransportEventsListenerOptions>,
  ) -> napi::Result<TransportEventsListener> {
    let TransportEventsListenerOptions { history, channel } = options.unwrap_or_default();
    let handler = into_handler(channel);

    let listener = build!(self.0.transport_events_listener().with(handler), history,)
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
      self.0.link_events_listener().with(handler),
      history,
      transport,
    )
    .await
    .map_napi_err()?;

    Ok(listener.into())
  }
}

#[derive(Clone, From, Into)]
#[napi]
pub struct Transport(z::session::Transport);

#[napi]
impl Transport {
  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.0.zid().to_string()
  }

  #[napi(getter)]
  pub fn whatami(&self) -> WhatAmI {
    self.0.whatami().into()
  }

  #[napi(getter)]
  pub fn is_qos(&self) -> bool {
    self.0.is_qos()
  }

  #[napi(getter)]
  pub fn is_multicast(&self) -> bool {
    self.0.is_multicast()
  }
}

#[derive(From, Into)]
#[napi]
pub struct TransportEvent(z::session::TransportEvent);

#[napi]
impl TransportEvent {
  #[napi(getter)]
  pub fn kind(&self) -> SampleKind {
    self.0.kind().into()
  }

  #[napi(getter)]
  pub fn transport(&self) -> Transport {
    self.0.transport().clone().into()
  }
}

#[derive(From)]
#[from(forward)]
#[napi]
pub struct TransportEventsListener(
  Declared<
    TransportEventsListener,
    z::session::TransportEventsListener<HandlerImpl<z::session::TransportEvent>>,
  >,
);

#[napi]
impl TransportEventsListener {
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    z::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }

  #[napi]
  pub async fn recv(&self) -> napi::Result<DeferredJs> {
    self.0.get()?.recv().await
  }

  #[napi]
  pub fn try_recv(&self) -> napi::Result<Option<DeferredJs>> {
    self.0.get()?.try_recv()
  }

  #[napi]
  pub fn stream(&self) -> napi::Result<Stream> {
    Ok(self.0.get()?.stream())
  }

  #[napi]
  pub fn handler(&self) -> napi::Result<Handler> {
    Ok(self.0.get()?.share())
  }
}

#[napi(object)]
pub struct LinkPriorities {
  pub min: u8,
  pub max: u8,
}

#[derive(From, Into)]
#[napi]
pub struct Link(z::session::Link);

#[napi]
impl Link {
  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.0.zid().to_string()
  }

  #[napi(getter)]
  pub fn src(&self) -> Locator {
    self.0.src().clone().into()
  }

  #[napi(getter)]
  pub fn dst(&self) -> Locator {
    self.0.dst().clone().into()
  }

  #[napi(getter)]
  pub fn group(&self) -> Option<Locator> {
    self.0.group().cloned().map(Locator::from)
  }

  #[napi(getter)]
  pub fn mtu(&self) -> u16 {
    self.0.mtu()
  }

  #[napi(getter)]
  pub fn is_streamed(&self) -> bool {
    self.0.is_streamed()
  }

  #[napi(getter)]
  pub fn interfaces(&self) -> Vec<String> {
    self.0.interfaces().to_vec()
  }

  #[napi(getter)]
  pub fn auth_identifier(&self) -> Option<String> {
    self.0.auth_identifier().map(|s| s.to_string())
  }

  #[napi(getter)]
  pub fn priorities(&self) -> Option<LinkPriorities> {
    self
      .0
      .priorities()
      .map(|(min, max)| LinkPriorities { min, max })
  }

  #[napi(getter)]
  pub fn reliability(&self) -> Option<Reliability> {
    self.0.reliability().map(Into::into)
  }
}

#[derive(From, Into)]
#[napi]
pub struct LinkEvent(z::session::LinkEvent);

#[napi]
impl LinkEvent {
  #[napi(getter)]
  pub fn kind(&self) -> SampleKind {
    self.0.kind().into()
  }

  #[napi(getter)]
  pub fn link(&self) -> Link {
    self.0.link().clone().into()
  }
}

#[derive(From)]
#[from(forward)]
#[napi]
pub struct LinkEventsListener(
  Declared<LinkEventsListener, z::session::LinkEventsListener<HandlerImpl<z::session::LinkEvent>>>,
);

#[napi]
impl LinkEventsListener {
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    z::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }

  #[napi]
  pub async fn recv(&self) -> napi::Result<DeferredJs> {
    self.0.get()?.recv().await
  }

  #[napi]
  pub fn try_recv(&self) -> napi::Result<Option<DeferredJs>> {
    self.0.get()?.try_recv()
  }

  #[napi]
  pub fn stream(&self) -> napi::Result<Stream> {
    Ok(self.0.get()?.stream())
  }

  #[napi]
  pub fn handler(&self) -> napi::Result<Handler> {
    Ok(self.0.get()?.share())
  }
}

#[derive(From, Into)]
#[napi]
pub struct Locator(z::config::Locator);

#[napi]
impl Locator {
  #[napi(constructor)]
  pub fn new(protocol: String, address: String, metadata: String) -> napi::Result<Self> {
    let inner = z::config::Locator::new(protocol, address, metadata).map_napi_err()?;
    Ok(inner.into())
  }

  #[napi(getter)]
  pub fn protocol(&self) -> String {
    self.0.protocol().as_str().to_string()
  }

  #[napi(getter)]
  pub fn address(&self) -> String {
    self.0.address().as_str().to_string()
  }

  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.0.as_str().to_string()
  }

  #[napi]
  pub fn metadata(&self) -> Metadata {
    self.0.to_endpoint().into()
  }

  #[napi]
  pub fn to_endpoint(&self) -> EndPoint {
    self.0.to_endpoint().into()
  }
}

#[derive(From, Into)]
#[napi]
pub struct Metadata(z::config::EndPoint);

#[napi]
impl Metadata {
  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.0.metadata().as_str().to_string()
  }

  #[napi]
  pub fn is_empty(&self) -> bool {
    self.0.metadata().is_empty()
  }

  #[napi]
  pub fn get(&self, key: String) -> Option<String> {
    self.0.metadata().get(&key).map(|value| value.to_string())
  }

  #[napi]
  pub fn values(&self, key: String) -> Vec<String> {
    self
      .0
      .metadata()
      .values(&key)
      .map(|value| value.to_string())
      .collect()
  }
}

#[derive(From, Into)]
#[napi]
pub struct EndPoint(z::config::EndPoint);

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
    let inner = z::config::EndPoint::new(protocol, address, metadata, config).map_napi_err()?;

    Ok(inner.into())
  }

  #[napi(getter)]
  pub fn protocol(&self) -> String {
    self.0.protocol().as_str().to_string()
  }

  #[napi(getter)]
  pub fn address(&self) -> String {
    self.0.address().as_str().to_string()
  }

  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.0.as_str().to_string()
  }

  #[napi]
  pub fn metadata(&self) -> Metadata {
    self.0.clone().into()
  }

  #[napi]
  pub fn config(&self) -> String {
    self.0.config().as_str().to_string()
  }

  #[napi]
  pub fn split(&self) -> EndPointParts {
    let (protocol, address, metadata, config) = self.0.split();
    EndPointParts {
      protocol: protocol.as_str().to_string(),
      address: address.as_str().to_string(),
      metadata: metadata.as_str().to_string(),
      config: config.as_str().to_string(),
    }
  }

  #[napi]
  pub fn to_locator(&self) -> Locator {
    self.0.to_locator().into()
  }
}
