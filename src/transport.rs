use std::sync::Arc;

use napi::bindgen_prelude::Either;
use napi_derive::napi;
use zenoh::handlers::{FifoChannelHandler, RingChannelHandler};
use zenoh::session::{
  Transport as ZTransport, TransportEvent as ZTransportEvent, TransportEventsListener as ZListener,
};

use crate::handlers::{FifoChannelHandlerTransportEvent, RingChannelHandlerTransportEvent};
use crate::sample::SampleKind;
use crate::whatami::WhatAmI;

/// A transport is a connection established to a remote zenoh node. Multiple
/// transports to the same node can coexist (e.g. a unicast and a multicast one).
/// Each transport carries one or more [`Link`](crate::link::Link)s.
///
/// Obtained from `SessionInfo.transports` or a [`TransportEvent`].
#[napi]
pub struct Transport {
  inner: ZTransport,
}

impl Transport {
  /// Internal constructor contract: wrap an owned `zenoh` value.
  pub(crate) fn from_inner(inner: ZTransport) -> Self {
    Transport { inner }
  }
}

#[napi]
impl Transport {
  /// The Zenoh id of the remote node, as a hex string.
  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.inner.zid().to_string()
  }

  /// The type of the remote node (Router, Peer or Client).
  #[napi(getter)]
  pub fn whatami(&self) -> WhatAmI {
    self.inner.whatami().into()
  }

  /// Whether this transport supports QoS.
  #[napi(getter)]
  pub fn is_qos(&self) -> bool {
    self.inner.is_qos()
  }

  /// Whether this transport is multicast.
  #[napi(getter)]
  pub fn is_multicast(&self) -> bool {
    self.inner.is_multicast()
  }
}

/// An event emitted when a transport is opened or closed. `kind` is `Put` when
/// the transport opened and `Delete` when it closed.
///
/// Delivered by a [`TransportEventsListener`].
#[napi]
pub struct TransportEvent {
  inner: ZTransportEvent,
}

impl TransportEvent {
  /// Internal constructor contract: wrap an owned `zenoh` value.
  pub(crate) fn from_inner(inner: ZTransportEvent) -> Self {
    TransportEvent { inner }
  }
}

#[napi]
impl TransportEvent {
  /// `Put` if the transport opened, `Delete` if it closed.
  #[napi(getter)]
  pub fn kind(&self) -> SampleKind {
    self.inner.kind().into()
  }

  /// The transport this event is about.
  #[napi(getter)]
  pub fn transport(&self) -> Transport {
    Transport::from_inner(self.inner.transport().clone())
  }
}

enum ListenerInner {
  Fifo(ZListener<FifoChannelHandler<ZTransportEvent>>),
  Ring(Arc<ZListener<RingChannelHandler<ZTransportEvent>>>),
}

/// A listener that notifies of transport lifecycle events (a transport opening
/// or closing). Declared via `SessionInfo.transportEventsListener`.
#[napi]
pub struct TransportEventsListener {
  // `None` once undeclared.
  inner: Option<ListenerInner>,
}

impl TransportEventsListener {
  pub(crate) fn from_fifo(listener: ZListener<FifoChannelHandler<ZTransportEvent>>) -> Self {
    TransportEventsListener {
      inner: Some(ListenerInner::Fifo(listener)),
    }
  }

  pub(crate) fn from_ring(listener: ZListener<RingChannelHandler<ZTransportEvent>>) -> Self {
    TransportEventsListener {
      inner: Some(ListenerInner::Ring(Arc::new(listener))),
    }
  }
}

#[napi]
impl TransportEventsListener {
  /// The receive end of the listener. A `FifoChannelHandler` or
  /// `RingChannelHandler` depending on the channel chosen at declare time.
  ///
  /// The handler is not iterable; iterate via `listener.handler.stream()`.
  #[napi(getter)]
  pub fn handler(
    &self,
  ) -> napi::Result<Either<FifoChannelHandlerTransportEvent, RingChannelHandlerTransportEvent>> {
    match self.inner.as_ref() {
      Some(ListenerInner::Fifo(listener)) => Ok(Either::A(
        FifoChannelHandlerTransportEvent::from_handler(listener.handler().clone()),
      )),
      Some(ListenerInner::Ring(arc)) => Ok(Either::B(RingChannelHandlerTransportEvent::from_arc(
        Arc::clone(arc),
      ))),
      None => Err(napi::Error::from_reason(
        "transport events listener has been undeclared",
      )),
    }
  }

  /// Undeclare this listener. Resolves once undeclaration completes; a second
  /// call is a no-op.
  ///
  /// For a ring listener still referenced by an outstanding handler, this drops
  /// our strong reference and lets the background drop undeclare it once the
  /// last handler is released.
  #[napi]
  pub async unsafe fn undeclare(&mut self) -> napi::Result<()> {
    match self.inner.take() {
      Some(ListenerInner::Fifo(listener)) => listener
        .undeclare()
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string())),
      Some(ListenerInner::Ring(arc)) => match Arc::try_unwrap(arc) {
        Ok(listener) => listener
          .undeclare()
          .await
          .map_err(|e| napi::Error::from_reason(e.to_string())),
        Err(_) => Ok(()),
      },
      None => Ok(()),
    }
  }
}
