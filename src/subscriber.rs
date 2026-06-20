use std::future::Future;
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::handlers::{DefaultHandler, FifoChannelHandler, RingChannelHandler};
use zenoh::pubsub::SubscriberBuilder;

use crate::error::to_napi_err;
use crate::handlers::{self, ChannelHandler, ChannelReceiver, ChannelType};
use crate::sample::{Locality, Sample};
use crate::session::EntityGlobalId;

type FifoSubscriber = zenoh::pubsub::Subscriber<Arc<FifoChannelHandler<zenoh::sample::Sample>>>;
type RingSubscriber = zenoh::pubsub::Subscriber<Arc<RingChannelHandler<zenoh::sample::Sample>>>;

/// The declared subscriber, kept alive (and undeclarable) regardless of which
/// channel kind backs it.
enum SubscriberInner {
  Fifo(FifoSubscriber),
  Ring(RingSubscriber),
}

impl SubscriberInner {
  fn key_expr(&self) -> String {
    match self {
      SubscriberInner::Fifo(subscriber) => subscriber.key_expr().as_str().to_string(),
      SubscriberInner::Ring(subscriber) => subscriber.key_expr().as_str().to_string(),
    }
  }

  fn id(&self) -> zenoh::session::EntityGlobalId {
    match self {
      SubscriberInner::Fifo(subscriber) => subscriber.id(),
      SubscriberInner::Ring(subscriber) => subscriber.id(),
    }
  }

  fn undeclare(self) -> Result<()> {
    use zenoh::Wait;
    match self {
      SubscriberInner::Fifo(subscriber) => subscriber.undeclare().wait().map_err(to_napi_err),
      SubscriberInner::Ring(subscriber) => subscriber.undeclare().wait().map_err(to_napi_err),
    }
  }
}

/// Options for [`Session::declareSubscriber`].
#[napi(object)]
pub struct SubscriberOptions {
  /// Restrict which publishers' samples are accepted (default: `Any`).
  pub allowed_origin: Option<Locality>,
  /// Channel handler (FIFO or Ring) backing delivery. Defaults to FIFO.
  pub handler: Option<ChannelHandler>,
}

/// A subscriber that delivers [`Sample`]s through a channel.
///
/// Consume it with `for await (const sample of subscriber)`, or pull samples
/// individually with `recv()` / `tryRecv()`. Iteration ends (yields `null`)
/// once the subscriber is undeclared — its buffered samples are dropped with the
/// handler, as in zenoh — or once the session/link closes and any buffered
/// samples have been drained.
#[napi(async_iterator)]
pub struct Subscriber {
  inner: Option<SubscriberInner>,
  /// Released together with `inner` on undeclare, so the handler (and any
  /// samples still buffered in it) is dropped exactly as zenoh's own `undeclare`
  /// does, rather than left draining after the subscriber is gone.
  receiver: Option<ChannelReceiver<zenoh::sample::Sample>>,
}

impl Subscriber {
  pub(crate) async fn declare(
    builder: SubscriberBuilder<'_, '_, DefaultHandler>,
    channel: Option<ChannelHandler>,
  ) -> Result<Self> {
    let (kind, capacity) = match channel {
      Some(channel) => (channel.kind, channel.capacity),
      None => (ChannelType::Fifo, None),
    };
    let (inner, receiver) = match kind {
      ChannelType::Fifo => {
        let (handler, receiver) = handlers::fifo_parts::<zenoh::sample::Sample>(capacity);
        let subscriber = builder.with(handler).await.map_err(to_napi_err)?;
        (SubscriberInner::Fifo(subscriber), receiver)
      }
      ChannelType::Ring => {
        let (handler, receiver) = handlers::ring_parts::<zenoh::sample::Sample>(capacity);
        let subscriber = builder.with(handler).await.map_err(to_napi_err)?;
        (SubscriberInner::Ring(subscriber), receiver)
      }
    };
    Ok(Self {
      inner: Some(inner),
      receiver: Some(receiver),
    })
  }

  fn get(&self) -> Result<&SubscriberInner> {
    self
      .inner
      .as_ref()
      .ok_or_else(|| Error::from_reason("subscriber has been undeclared"))
  }
}

#[napi]
impl Subscriber {
  /// Wait for the next sample, resolving to `null` once the subscriber is
  /// undeclared, or once it closes and all buffered samples have been drained.
  #[napi]
  pub async fn recv(&self) -> Result<Option<Sample>> {
    let receiver = self.receiver.clone();
    match receiver {
      Some(receiver) => Ok(receiver.recv().await.map(Sample::new)),
      None => Ok(None),
    }
  }

  /// Return a buffered sample if one is immediately available, or `null` if the
  /// channel is currently empty. Throws once the subscriber has disconnected
  /// (undeclared, or the session closed and all buffered samples drained),
  /// letting a polling loop tell "nothing yet" apart from "closed".
  #[napi]
  pub fn try_recv(&self) -> Result<Option<Sample>> {
    match &self.receiver {
      Some(receiver) => receiver
        .try_recv()
        .map(|sample| sample.map(Sample::new))
        .map_err(to_napi_err),
      None => Err(Error::from_reason("subscriber has been undeclared")),
    }
  }

  /// Undeclare the subscriber. Iteration / `recv` then end and `tryRecv` throws;
  /// any buffered samples are dropped with the handler. Resolves synchronously.
  #[napi]
  pub fn undeclare(&mut self) -> Result<()> {
    // Release the receiver with the declaration: zenoh drops the handler (and
    // anything still buffered in it) as part of undeclaring, so mirror that
    // instead of leaving a FIFO buffer draining after the subscriber is gone.
    self.receiver = None;
    match self.inner.take() {
      Some(inner) => inner.undeclare(),
      None => Ok(()),
    }
  }

  /// The key expression this subscriber is subscribed to.
  #[napi(getter)]
  pub fn key_expr(&self) -> Result<String> {
    Ok(self.get()?.key_expr())
  }

  /// This subscriber's globally-unique entity id.
  #[napi(getter)]
  pub fn id(&self) -> Result<EntityGlobalId> {
    Ok(EntityGlobalId::from_zenoh(self.get()?.id()))
  }
}

#[napi]
impl AsyncGenerator for Subscriber {
  type Yield = Sample;
  type Next = ();
  type Return = ();

  fn next(
    &mut self,
    _value: Option<Self::Next>,
  ) -> impl Future<Output = Result<Option<Self::Yield>>> + Send + 'static {
    let receiver = self.receiver.clone();
    async move {
      match receiver {
        Some(receiver) => Ok(receiver.recv().await.map(Sample::new)),
        None => Ok(None),
      }
    }
  }
}
