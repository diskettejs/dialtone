use napi_derive::napi;
use zenoh::handlers::{FifoChannel, RingChannel};

use crate::handlers::{ChannelKind, DEFAULT_CHANNEL_CAPACITY};
use crate::link::{Link, LinkEventsListener};
use crate::locator::Locator;
use crate::options::{LinkEventsListenerOptions, TransportEventsListenerOptions};
use crate::transport::{Transport, TransportEventsListener};

/// The connectivity sub-API for a session: who it is connected to (transports,
/// links), its own and its neighbours' Zenoh ids, and listeners for transport /
/// link lifecycle events. Reached via `Session.info()`.
#[napi]
pub struct SessionInfo {
  session: zenoh::Session,
}

impl SessionInfo {
  pub(crate) fn from_session(session: zenoh::Session) -> Self {
    SessionInfo { session }
  }
}

#[napi]
impl SessionInfo {
  /// This session's Zenoh id, as a hex string.
  #[napi]
  pub async fn zid(&self) -> String {
    self.session.info().zid().await.to_string()
  }

  /// The Zenoh ids of the routers this session is currently connected to (or of
  /// the current router, if running inside one), as hex strings.
  #[napi]
  pub async fn routers_zid(&self) -> Vec<String> {
    self
      .session
      .info()
      .routers_zid()
      .await
      .map(|zid| zid.to_string())
      .collect()
  }

  /// The Zenoh ids of the peers this session is currently connected to, as hex
  /// strings.
  #[napi]
  pub async fn peers_zid(&self) -> Vec<String> {
    self
      .session
      .info()
      .peers_zid()
      .await
      .map(|zid| zid.to_string())
      .collect()
  }

  /// The locators this session is listening on.
  #[napi]
  pub async fn locators(&self) -> Vec<Locator> {
    self
      .session
      .info()
      .locators()
      .await
      .into_iter()
      .map(Locator::from_inner)
      .collect()
  }

  /// The currently-open transports (connections to remote nodes).
  #[napi]
  pub async fn transports(&self) -> Vec<Transport> {
    self
      .session
      .info()
      .transports()
      .await
      .map(Transport::from_inner)
      .collect()
  }

  /// The currently-established links across all transports.
  #[napi]
  pub async fn links(&self) -> Vec<Link> {
    self
      .session
      .info()
      .links()
      .await
      .map(Link::from_inner)
      .collect()
  }

  /// Declares a listener for transport lifecycle events (a transport opening or
  /// closing). The `handler` option chooses the channel (default: FIFO of
  /// [`DEFAULT_CHANNEL_CAPACITY`]); `history` replays the currently-open
  /// transports on declaration.
  #[napi]
  pub async fn transport_events_listener(
    &self,
    options: Option<TransportEventsListenerOptions>,
  ) -> napi::Result<TransportEventsListener> {
    let handler_cfg = options.as_ref().and_then(|o| o.handler.as_ref());
    let capacity = handler_cfg
      .and_then(|c| c.capacity)
      .map(|c| c as usize)
      .unwrap_or(DEFAULT_CHANNEL_CAPACITY);
    let is_ring = handler_cfg.is_some_and(|c| matches!(c.kind, ChannelKind::Ring));
    let history = options.as_ref().and_then(|o| o.history).unwrap_or(false);

    let info = self.session.info();
    let builder = info.transport_events_listener().history(history);

    if is_ring {
      let listener = builder
        .with(RingChannel::new(capacity))
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
      Ok(TransportEventsListener::from_ring(listener))
    } else {
      let listener = builder
        .with(FifoChannel::new(capacity))
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
      Ok(TransportEventsListener::from_fifo(listener))
    }
  }

  /// Declares a listener for link lifecycle events (a link being added or
  /// removed). The `handler` option chooses the channel (default: FIFO of
  /// [`DEFAULT_CHANNEL_CAPACITY`]); `history` replays the currently-established
  /// links on declaration.
  #[napi]
  pub async fn link_events_listener(
    &self,
    options: Option<LinkEventsListenerOptions>,
  ) -> napi::Result<LinkEventsListener> {
    let handler_cfg = options.as_ref().and_then(|o| o.handler.as_ref());
    let capacity = handler_cfg
      .and_then(|c| c.capacity)
      .map(|c| c as usize)
      .unwrap_or(DEFAULT_CHANNEL_CAPACITY);
    let is_ring = handler_cfg.is_some_and(|c| matches!(c.kind, ChannelKind::Ring));
    let history = options.as_ref().and_then(|o| o.history).unwrap_or(false);

    let info = self.session.info();
    let builder = info.link_events_listener().history(history);

    if is_ring {
      let listener = builder
        .with(RingChannel::new(capacity))
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
      Ok(LinkEventsListener::from_ring(listener))
    } else {
      let listener = builder
        .with(FifoChannel::new(capacity))
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
      Ok(LinkEventsListener::from_fifo(listener))
    }
  }
}
