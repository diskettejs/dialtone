use derive_more::From;
use napi::bindgen_prelude::AsyncGenerator;
use napi_derive::napi;
use zenoh as z;
use zenoh_ext::{AdvancedPublisherBuilderExt, AdvancedSubscriberBuilderExt};

use crate::{
  bytes::BytesBuffer,
  config::{Config, EntityGlobalId, SessionConfig, WhatAmI},
  key_expr::{KeyExpr, KeyExprArg},
  liveliness::Liveliness,
  options::{
    DeleteOptions, GetOptions, LinkEventsListenerOptions, PublisherOptions, PutOptions,
    QuerierOptions, QueryableOptions, SubscriberOptions, TransportEventsListenerOptions,
  },
  pubsub::{Publisher, Subscriber},
  qos::Reliability,
  query::{Querier, Queryable, Replies, Selector, SelectorArg},
  sample::SampleKind,
  time::Timestamp,
  utils::{Declared, MapNapiErr, duration_ms, fifo},
};

/// The main component of Zenoh, holding this node's connection to the network.
///
/// A session is opened with {@link Session.open} and declares the other Zenoh entities:
/// publishers, subscribers, queriers, queryables and liveliness tokens. Those entities
/// have a lifetime of their own, but they stop working once the session is closed.
#[napi]
#[derive(From)]
pub struct Session(z::Session);

#[napi]
impl Session {
  /// Opens a session with the given configuration.
  ///
  /// @throws If the session could not be opened.
  #[napi(factory)]
  pub async fn open(config: &Config) -> napi::Result<Self> {
    let cfg = config.as_ref().clone();
    let session = z::open(cfg).await.map_napi_err()?;
    Ok(session.into())
  }

  /// The Zenoh identifier of this session.
  ///
  /// Shortcut for {@link SessionInfo.zid}.
  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.0.zid().to_string()
  }

  /// The identifier of this session as a Zenoh entity.
  #[napi(getter)]
  pub fn id(&self) -> EntityGlobalId {
    self.0.id().into()
  }

  /// Whether this session has been closed.
  #[napi(getter)]
  pub fn is_closed(&self) -> bool {
    self.0.is_closed()
  }

  /// Returns a timestamp carrying the current time and this session's Zenoh identifier.
  #[napi]
  pub fn new_timestamp(&self) -> Timestamp {
    self.0.new_timestamp().into()
  }

  /// Returns information about this session and the network around it.
  #[napi]
  pub fn info(&self) -> SessionInfo {
    self.0.info().into()
  }

  /// Returns the configuration this session is currently running with.
  #[napi]
  pub fn config(&self) -> SessionConfig {
    self.0.config().into()
  }

  /// Closes this session.
  ///
  /// Every subscriber and queryable declared by this session stops receiving data, and
  /// further attempts to publish or query with the session or its publishers fail.
  /// Undeclaring an entity after the session is closed is a no-op.
  ///
  /// @throws If the session could not be closed.
  #[napi]
  pub async fn close(&self) -> napi::Result<()> {
    self.0.close().await.map_napi_err()
  }

  /// Publishes a payload on the resources matching a key expression.
  ///
  /// Shortcut for declaring a publisher with {@link Session.declarePublisher} and calling
  /// {@link Publisher.put} on it.
  ///
  /// @throws If the key expression is invalid or the session is closed.
  #[napi]
  pub async fn put(
    &self,
    key_expr: KeyExprArg<'_>,
    payload: BytesBuffer,
    options: Option<PutOptions>,
  ) -> napi::Result<()> {
    let expr = KeyExpr::try_from(key_expr)?;
    let PutOptions {
      encoding,
      congestion_control,
      priority,
      express,
      reliability,
      allowed_destination,
      attachment,
    } = options.unwrap_or_default();
    let session = self.0.clone();
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

    if let Some(attachment) = attachment {
      builder = builder.attachment(attachment);
    }

    builder.await.map_napi_err()
  }

  /// Queries the queryables matching a selector.
  ///
  /// Shortcut for declaring a querier with {@link Session.declareQuerier} and calling
  /// {@link Querier.get} on it.
  ///
  /// Replies are guaranteed to carry a key expression that matches the selector.
  ///
  /// @returns The replies to this query.
  /// @throws If the selector is invalid or the session is closed.
  #[napi]
  pub async fn get(
    &self,
    selector: SelectorArg<'_>,
    options: Option<GetOptions>,
  ) -> napi::Result<Replies> {
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
      // cancellation_token,
      channel_capacity,
    } = options.unwrap_or_default();
    let selector = Selector::resolve(selector, parameters)?;
    let timeout = duration_ms(timeout)?;
    let handler = fifo(channel_capacity);
    let session = self.0.clone();

    let mut builder = session.get(selector).with(handler);

    if let Some(target) = target {
      builder = builder.target(target.into());
    }

    if let Some(consolidation) = consolidation {
      builder = builder.consolidation(consolidation);
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

    let receiver = builder.await.map_napi_err()?;

    Ok(receiver.into())
  }

  /// Publishes a delete on the resources matching a key expression.
  ///
  /// Shortcut for declaring a publisher with {@link Session.declarePublisher} and calling
  /// {@link Publisher.delete} on it.
  ///
  /// @throws If the key expression is invalid or the session is closed.
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
      attachment,
    } = options.unwrap_or_default();
    let session = self.0.clone();
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

    if let Some(attachment) = attachment {
      builder = builder.attachment(attachment);
    }

    builder.await.map_napi_err()
  }

  /// Returns the liveliness interface of this session, used to declare liveliness tokens
  /// and to subscribe to or query the tokens alive in the network.
  #[napi]
  pub fn liveliness(&self) -> Liveliness {
    self.0.clone().into()
  }

  /// Informs Zenoh that a key expression will be used repeatedly, so that it optimizes its
  /// transmission.
  ///
  /// @returns The declared key expression, to be used in place of the original one.
  /// @throws If the key expression is invalid or the session is closed.
  #[napi]
  pub async fn declare_keyexpr(&self, key_expr: KeyExprArg<'_>) -> napi::Result<KeyExpr> {
    let value = KeyExpr::try_from(key_expr)?;

    Ok(self.0.declare_keyexpr(value).await.map_napi_err()?.into())
  }

  /// Declares a querier that repeatedly queries the resources matching a key expression.
  ///
  /// @throws If the key expression is invalid or the session is closed.
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

    let mut builder = self.0.declare_querier(expr);

    if let Some(target) = target {
      builder = builder.target(target.into());
    }

    if let Some(consolidation) = consolidation {
      builder = builder.consolidation(consolidation);
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

    if let Some(accept_replies) = accept_replies {
      builder = builder.accept_replies(accept_replies.into());
    }

    let zquerier = builder.await.map_napi_err()?;

    Ok(zquerier.into())
  }

  /// Declares a queryable that answers the queries matching a key expression.
  ///
  /// @throws If the key expression is invalid or the session is closed.
  #[napi]
  pub async fn declare_queryable(
    &self,
    key_expr: KeyExprArg<'_>,
    options: Option<QueryableOptions>,
  ) -> napi::Result<Queryable> {
    let QueryableOptions {
      allowed_origin,
      complete,
      channel_capacity,
    } = options.unwrap_or_default();
    let expr = KeyExpr::try_from(key_expr)?;
    let handler = fifo(channel_capacity);
    let mut builder = self.0.declare_queryable(expr).with(handler);

    if let Some(allowed_origin) = allowed_origin {
      builder = builder.allowed_origin(allowed_origin.into());
    }

    if let Some(complete) = complete {
      builder = builder.complete(complete);
    }

    let queryable = builder.await.map_napi_err()?;

    Ok(queryable.into())
  }

  /// Declares a subscriber that receives the data published on the resources matching a
  /// key expression.
  ///
  /// @throws If the key expression is invalid or the session is closed.
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
      channel_capacity,
    } = options.unwrap_or_default();
    let query_timeout = duration_ms(query_timeout_ms)?;
    let handler = fifo(channel_capacity);
    let session = self.0.clone();

    let mut builder = session
      .declare_subscriber(key_expr)
      .advanced()
      .with(handler);

    if let Some(allowed_origin) = allowed_origin {
      builder = builder.allowed_origin(allowed_origin.into());
    }

    if let Some(history) = history {
      builder = builder.history(history.into());
    }

    if let Some(recovery) = recovery {
      builder = builder.recovery(match recovery {
        napi::Either::A(periodic) => periodic.into(),
        napi::Either::B(heartbeat) => heartbeat.into(),
      });
    }

    if let Some(query_timeout) = query_timeout {
      builder = builder.query_timeout(query_timeout);
    }

    if let Some(subscriber_detection_metadata) = subscriber_detection_metadata {
      builder = builder.subscriber_detection_metadata(subscriber_detection_metadata);
    }

    if subscriber_detection == Some(true) {
      builder = builder.subscriber_detection();
    }

    let subscriber = builder.await.map_napi_err()?;

    Ok(subscriber.into())
  }

  /// Declares a publisher that writes to the resources matching a key expression.
  ///
  /// @throws If the key expression is invalid or the session is closed.
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
      reliability,
      sample_miss_detection,
    } = options.unwrap_or_default();

    let mut builder = self.0.declare_publisher(expr).advanced();

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

    if let Some(sample_miss_detection) = sample_miss_detection {
      builder = builder.sample_miss_detection(sample_miss_detection.into());
    }

    if publisher_detection == Some(true) {
      builder = builder.publisher_detection();
    }

    let publisher = builder.await.map_napi_err()?;

    Ok(publisher.into())
  }
}

/// Information about a session and the Zenoh network around it.
///
/// This covers the identifier of the session itself, the identifiers of the routers and
/// peers it is connected to, and the transports and links currently established.
#[napi]
#[derive(From)]
pub struct SessionInfo(z::session::SessionInfo);

#[napi]
impl SessionInfo {
  /// Returns the Zenoh identifier of this session.
  #[napi]
  pub async fn zid(&self) -> String {
    self.0.zid().await.to_string()
  }

  /// Returns the Zenoh identifiers of the routers this process is currently connected to,
  /// or the identifier of the current router when this code runs inside one.
  #[napi]
  pub async fn routers_zid(&self) -> Vec<String> {
    self
      .0
      .routers_zid()
      .await
      .map(|zid| zid.to_string())
      .collect()
  }

  /// Returns the Zenoh identifiers of the peers this process is currently connected to.
  #[napi]
  pub async fn peers_zid(&self) -> Vec<String> {
    self
      .0
      .peers_zid()
      .await
      .map(|zid| zid.to_string())
      .collect()
  }

  /// Returns the currently open transports, i.e. the connections to other Zenoh nodes.
  #[napi]
  pub async fn transports(&self) -> Vec<Transport> {
    self.0.transports().await.map(Transport::from).collect()
  }

  /// Returns the links established across all transports.
  #[napi]
  pub async fn links(&self) -> Vec<Link> {
    self.0.links().await.map(Link::from).collect()
  }

  /// Declares a listener notified whenever a transport is opened or closed.
  ///
  /// @throws If the listener could not be declared.
  #[napi]
  pub async fn transport_events_listener(
    &self,
    options: Option<TransportEventsListenerOptions>,
  ) -> napi::Result<TransportEventsListener> {
    let TransportEventsListenerOptions {
      history,
      channel_capacity,
    } = options.unwrap_or_default();
    let handler = fifo(channel_capacity);
    let mut builder = self.0.transport_events_listener().with(handler);

    if let Some(history) = history {
      builder = builder.history(history);
    }

    let listener = builder.await.map_napi_err()?;

    Ok(listener.into())
  }

  /// Declares a listener notified whenever a link is added or removed.
  ///
  /// @throws If the listener could not be declared.
  #[napi]
  pub async fn link_events_listener(
    &self,
    options: Option<LinkEventsListenerOptions>,
  ) -> napi::Result<LinkEventsListener> {
    let LinkEventsListenerOptions {
      history,
      channel_capacity,
    } = options.unwrap_or_default();
    let handler = fifo(channel_capacity);
    let mut builder = self.0.link_events_listener().with(handler);

    if let Some(history) = history {
      builder = builder.history(history);
    }

    let listener = builder.await.map_napi_err()?;

    Ok(listener.into())
  }
}

/// A connection established to a remote Zenoh node.
///
/// Several transports to the same node can coexist; a unicast and a multicast transport to
/// the same node are both possible. Each transport carries one or more {@link `Link`}s, the
/// data links actually established with the various protocols.
#[napi]
#[derive(Clone, From)]
pub struct Transport(z::session::Transport);

#[napi]
impl Transport {
  /// The Zenoh identifier of the remote node.
  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.0.zid().to_string()
  }

  /// The kind of the remote node: router, peer or client.
  #[napi(getter)]
  pub fn whatami(&self) -> WhatAmI {
    self.0.whatami().into()
  }

  /// Whether this transport supports `QoS`.
  #[napi(getter)]
  pub fn is_qos(&self) -> bool {
    self.0.is_qos()
  }

  /// Whether this transport is multicast.
  #[napi(getter)]
  pub fn is_multicast(&self) -> bool {
    self.0.is_multicast()
  }
}

/// An event reported when a transport is opened or closed.
#[napi]
#[derive(From)]
pub struct TransportEvent(z::session::TransportEvent);

#[napi]
impl TransportEvent {
  /// Whether the transport was opened (`Put`) or closed (`Delete`).
  #[napi(getter)]
  pub fn kind(&self) -> SampleKind {
    self.0.kind().into()
  }

  /// The transport this event is about.
  #[napi(getter)]
  pub fn transport(&self) -> Transport {
    self.0.transport().clone().into()
  }
}

/// A listener receiving the transport events of a session.
#[napi]
#[derive(From)]
#[from(forward)]
pub struct TransportEventsListener(
  Declared<
    z::session::TransportEventsListener<
      z::handlers::FifoChannelHandler<z::session::TransportEvent>,
    >,
  >,
);

#[napi]
impl TransportEventsListener {
  /// Undeclares this listener and stops receiving transport events.
  ///
  /// @throws If this listener has already been undeclared.
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    z::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }

  /// Returns a stream of the transport events delivered to this listener.
  ///
  /// @throws If this listener has already been undeclared.
  #[napi]
  pub fn receive(&self) -> napi::Result<TransportEventIter> {
    let handler = self.0.get()?.handler().clone();

    Ok(handler.into())
  }
}

/// A stream of the transport events reported to a listener.
#[napi(async_iterator)]
#[derive(From)]
pub struct TransportEventIter(z::handlers::FifoChannelHandler<z::session::TransportEvent>);

#[napi]
impl AsyncGenerator for TransportEventIter {
  type Yield = TransportEvent;
  type Next = ();
  type Return = ();

  fn next(
    &mut self,
    _value: Option<Self::Next>,
  ) -> impl Future<Output = napi::Result<Option<Self::Yield>>> + Send + 'static {
    let receiver = self.0.clone();

    async move {
      match receiver.recv_async().await {
        Ok(event) => Ok(Some(event.into())),
        Err(_) => Ok(None),
      }
    }
  }
}

/// The inclusive range of priorities a link is used for.
///
/// The numeric values correspond to {@link `Priority`}, plus `0` for the control priority,
/// which that enum does not expose. The lower the value, the higher the priority.
#[napi(object)]
pub struct LinkPriorities {
  /// Lowest numeric value of the range, i.e. the highest priority the link carries.
  pub min: u8,
  /// Highest numeric value of the range, i.e. the lowest priority the link carries.
  pub max: u8,
}

/// A concrete data link within a {@link `Transport`}.
///
/// Zenoh can establish several links to the same remote node using different protocols,
/// e.g. TCP, UDP or QUIC.
#[napi]
#[derive(From)]
pub struct Link(z::session::Link);

#[napi]
impl Link {
  /// The Zenoh identifier of the transport this link belongs to.
  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.0.zid().to_string()
  }

  /// The source locator, i.e. the local end of this link.
  #[napi(getter)]
  pub fn src(&self) -> Locator {
    self.0.src().clone().into()
  }

  /// The destination locator, i.e. the remote end of this link.
  #[napi(getter)]
  pub fn dst(&self) -> Locator {
    self.0.dst().clone().into()
  }

  /// The group locator of a multicast link, or `null` when the link is not multicast.
  #[napi(getter)]
  pub fn group(&self) -> Option<Locator> {
    self.0.group().cloned().map(Locator::from)
  }

  /// The maximum transmission unit of this link, in bytes.
  #[napi(getter)]
  pub fn mtu(&self) -> u16 {
    self.0.mtu()
  }

  /// Whether this link's protocol is stream-oriented.
  #[napi(getter)]
  pub fn is_streamed(&self) -> bool {
    self.0.is_streamed()
  }

  /// The network interfaces associated with this link.
  #[napi(getter)]
  pub fn interfaces(&self) -> Vec<String> {
    self.0.interfaces().to_vec()
  }

  /// The authentication identifier of this link, or `null` when its protocol does not
  /// provide one.
  #[napi(getter)]
  pub fn auth_identifier(&self) -> Option<String> {
    self
      .0
      .auth_identifier()
      .map(std::string::ToString::to_string)
  }

  /// The range of priorities this link is used for, or `null` when its transport does not
  /// support `QoS`.
  #[napi(getter)]
  pub fn priorities(&self) -> Option<LinkPriorities> {
    self
      .0
      .priorities()
      .map(|(min, max)| LinkPriorities { min, max })
  }

  /// The reliability of this link, or `null` when its transport does not support `QoS`.
  #[napi(getter)]
  pub fn reliability(&self) -> Option<Reliability> {
    self.0.reliability().map(Into::into)
  }
}

/// An event reported when a link is added or removed.
#[napi]
#[derive(From)]
pub struct LinkEvent(z::session::LinkEvent);

#[napi]
impl LinkEvent {
  /// Whether the link was added (`Put`) or removed (`Delete`).
  #[napi(getter)]
  pub fn kind(&self) -> SampleKind {
    self.0.kind().into()
  }

  /// The link this event is about.
  #[napi(getter)]
  pub fn link(&self) -> Link {
    self.0.link().clone().into()
  }
}

/// A listener receiving the link events of a session.
#[napi]
#[derive(From)]
#[from(forward)]
pub struct LinkEventsListener(
  Declared<z::session::LinkEventsListener<z::handlers::FifoChannelHandler<z::session::LinkEvent>>>,
);

#[napi]
impl LinkEventsListener {
  /// Undeclares this listener and stops receiving link events.
  ///
  /// @throws If this listener has already been undeclared.
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    z::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }

  /// Returns a stream of the link events delivered to this listener.
  ///
  /// @throws If this listener has already been undeclared.
  #[napi]
  pub fn receive(&self) -> napi::Result<LinkEventIter> {
    let handler = self.0.get()?.handler().clone();

    Ok(handler.into())
  }
}

/// A stream of the link events reported to a listener.
#[napi(async_iterator)]
#[derive(From)]
pub struct LinkEventIter(z::handlers::FifoChannelHandler<z::session::LinkEvent>);

#[napi]
impl AsyncGenerator for LinkEventIter {
  type Yield = LinkEvent;
  type Next = ();
  type Return = ();

  fn next(
    &mut self,
    _value: Option<Self::Next>,
  ) -> impl Future<Output = napi::Result<Option<Self::Yield>>> + Send + 'static {
    let receiver = self.0.clone();

    async move {
      match receiver.recv_async().await {
        Ok(event) => Ok(Some(event.into())),
        Err(_) => Ok(None),
      }
    }
  }
}

/// An address at which a Zenoh node can be reached, in the canonical form
/// `<protocol>/<address>[?<metadata>]`.
///
/// The metadata part is a `;`-separated list of `<key>=<value>` pairs, sorted
/// alphabetically by key.
#[napi]
#[derive(From)]
pub struct Locator(z::config::Locator);

#[napi]
impl Locator {
  /// Builds a locator from its protocol, address and metadata parts.
  ///
  /// @throws If the parts do not form a valid locator.
  #[napi(constructor)]
  pub fn new(protocol: String, address: String, metadata: String) -> napi::Result<Self> {
    let inner = z::config::Locator::new(protocol, address, metadata).map_napi_err()?;
    Ok(inner.into())
  }

  /// The protocol part of this locator, e.g. `tcp`.
  #[napi(getter)]
  pub fn protocol(&self) -> String {
    self.0.protocol().as_str().to_string()
  }

  /// The address part of this locator, e.g. `127.0.0.1:7447`.
  #[napi(getter)]
  pub fn address(&self) -> String {
    self.0.address().as_str().to_string()
  }

  /// Returns this locator in its canonical string form.
  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.0.as_str().to_string()
  }

  /// Returns the metadata part of this locator.
  #[napi]
  pub fn metadata(&self) -> Metadata {
    self.0.to_endpoint().into()
  }

  /// Returns this locator as an endpoint with an empty configuration part.
  #[napi]
  pub fn to_endpoint(&self) -> EndPoint {
    self.0.to_endpoint().into()
  }
}

/// The metadata part of a {@link `Locator`} or an {@link `EndPoint`}.
///
/// Metadata is a `;`-separated list of `<key>=<value>` pairs. Zenoh reads `prio`, an
/// inclusive priority range such as `1-3`, and `rel`, either `0` for best effort or `1` for
/// reliable; both are used to select the link a message is sent on.
#[napi]
#[derive(From)]
pub struct Metadata(z::config::EndPoint);

#[napi]
impl Metadata {
  /// Returns this metadata in its `<key>=<value>;...` string form.
  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.0.metadata().as_str().to_string()
  }

  /// Returns `true` when this metadata carries no pair.
  #[napi]
  pub fn is_empty(&self) -> bool {
    self.0.metadata().is_empty()
  }

  /// Returns the value associated with `key`, or `null` when the key is absent.
  #[napi]
  pub fn get(&self, key: String) -> Option<String> {
    self
      .0
      .metadata()
      .get(&key)
      .map(std::string::ToString::to_string)
  }

  /// Returns the values associated with `key`, splitting the value on `|`.
  ///
  /// @returns The individual values, or an empty array when the key is absent.
  #[napi]
  pub fn values(&self, key: String) -> Vec<String> {
    self
      .0
      .metadata()
      .values(&key)
      .map(std::string::ToString::to_string)
      .collect()
  }
}

/// A {@link `Locator`} extended with a configuration part, in the canonical form
/// `<protocol>/<address>[?<metadata>][#<config>]`.
///
/// The configuration part is a `;`-separated list of `<key>=<value>` pairs, sorted
/// alphabetically by key. It configures aspects of the endpoint such as the interface to
/// listen on or to connect from.
#[napi]
#[derive(From)]
pub struct EndPoint(z::config::EndPoint);

/// The parts an {@link `EndPoint`} is made of.
#[napi(object)]
pub struct EndPointParts {
  /// The protocol part, e.g. `tcp`.
  pub protocol: String,
  /// The address part, e.g. `127.0.0.1:7447`.
  pub address: String,
  /// The metadata part, in its `<key>=<value>;...` string form.
  pub metadata: String,
  /// The configuration part, in its `<key>=<value>;...` string form.
  pub config: String,
}

#[napi]
impl EndPoint {
  /// Builds an endpoint from its protocol, address, metadata and configuration parts.
  ///
  /// @throws If the parts do not form a valid endpoint.
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

  /// The protocol part of this endpoint, e.g. `tcp`.
  #[napi(getter)]
  pub fn protocol(&self) -> String {
    self.0.protocol().as_str().to_string()
  }

  /// The address part of this endpoint, e.g. `127.0.0.1:7447`.
  #[napi(getter)]
  pub fn address(&self) -> String {
    self.0.address().as_str().to_string()
  }

  /// Returns this endpoint in its canonical string form.
  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.0.as_str().to_string()
  }

  /// Returns the metadata part of this endpoint.
  #[napi]
  pub fn metadata(&self) -> Metadata {
    self.0.clone().into()
  }

  /// Returns the configuration part of this endpoint, in its `<key>=<value>;...` string
  /// form.
  #[napi]
  pub fn config(&self) -> String {
    self.0.config().as_str().to_string()
  }

  /// Returns the protocol, address, metadata and configuration parts of this endpoint.
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

  /// Returns this endpoint as a locator, dropping its configuration part.
  #[napi]
  pub fn to_locator(&self) -> Locator {
    self.0.to_locator().into()
  }
}
