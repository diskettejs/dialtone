use crate::error::zerr;
use crate::sample::{Sample, to_js_sample};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::future::Future;
use std::sync::Arc;
use zenoh::handlers::{
  Callback, FifoChannel, FifoChannelHandler, IntoHandler, RingChannel, RingChannelHandler,
};

/// Zenoh's default reception-channel capacity (`API_DATA_RECEPTION_CHANNEL_SIZE`).
const DEFAULT_CAPACITY: usize = 256;

/// The handler is built up front and its callback is handed to the subscriber, so
/// every subscriber resolves to the same `()`-handler type regardless of channel.
type SampleSubscriber = zenoh::pubsub::Subscriber<()>;

/// Which channel handler buffers samples between the network and the consumer.
#[napi(string_enum = "lowercase")]
pub enum ChannelHandler {
  /// Back-pressures the network when full; a slow consumer can stall the Zenoh thread but never loses samples.
  Fifo,
  /// Keeps only the most recent samples and drops the oldest when full; a slow consumer loses old data instead of stalling the network.
  Ring,
}

/// Configures the channel that buffers a subscriber's samples.
#[napi(object)]
pub struct HandlerOptions {
  /// Buffering strategy. Defaults to `fifo`.
  #[napi(js_name = "type")]
  pub kind: Option<ChannelHandler>,
  /// Channel capacity (max buffered samples). Defaults to 256.
  pub capacity: Option<u32>,
}

/// Options for `declareSubscriber`.
#[napi(object)]
pub struct SubscriberOptions {
  /// The channel handler that buffers incoming samples. Defaults to a `fifo`
  /// channel with capacity 256.
  pub handler: Option<HandlerOptions>,
}

/// The receiver side of the chosen channel. Cloned into the `'static` `next()` /
/// `receive()` futures so they don't borrow `self`. The FIFO receiver is a cheap
/// `flume` clone; the ring receiver isn't `Clone`, so it's shared behind an `Arc`.
#[derive(Clone)]
pub(crate) enum Receiver {
  Fifo(FifoChannelHandler<zenoh::sample::Sample>),
  Ring(Arc<RingChannelHandler<zenoh::sample::Sample>>),
}

impl Receiver {
  /// Await the next sample, or `None` once all senders are gone (subscription
  /// closed/undeclared).
  async fn recv(&self) -> Option<Sample> {
    let received = match self {
      Receiver::Fifo(handler) => handler.recv_async().await,
      Receiver::Ring(handler) => handler.recv_async().await,
    };
    received.ok().map(to_js_sample)
  }
}

/// Build the channel handler from JS options, returning the callback to hand to
/// the subscriber and the receiver to keep for consumption. Keeping the receiver
/// (rather than letting the subscriber own its handler) is what lets both channel
/// kinds collapse to a single `Subscriber<()>` type.
pub(crate) fn build_handler(
  options: Option<HandlerOptions>,
) -> (Callback<zenoh::sample::Sample>, Receiver) {
  let (kind, capacity) = match options {
    Some(HandlerOptions { kind, capacity }) => (
      kind.unwrap_or(ChannelHandler::Fifo),
      capacity.map(|c| c as usize).unwrap_or(DEFAULT_CAPACITY),
    ),
    None => (ChannelHandler::Fifo, DEFAULT_CAPACITY),
  };
  match kind {
    ChannelHandler::Fifo => {
      let (callback, handler) = FifoChannel::new(capacity).into_handler();
      (callback, Receiver::Fifo(handler))
    }
    ChannelHandler::Ring => {
      let (callback, handler) = RingChannel::new(capacity).into_handler();
      (callback, Receiver::Ring(Arc::new(handler)))
    }
  }
}

/// A subscriber declared on a session. It is async-iterable — consume samples with
/// `for await (const sample of sub) { ... }` — or pull them one at a time with
/// `receive()`.
#[napi(async_iterator)]
pub struct Subscriber {
  key_expr: String,
  // Kept alive so the subscription stays declared; behind a Mutex so `undeclare`
  // (and the iterator's `complete`) can take it through a shared `&self`.
  subscriber: std::sync::Mutex<Option<SampleSubscriber>>,
  handler: Receiver,
}

impl Subscriber {
  pub(crate) fn new(subscriber: SampleSubscriber, handler: Receiver) -> Self {
    let key_expr = subscriber.key_expr().to_string();
    Self {
      key_expr,
      subscriber: std::sync::Mutex::new(Some(subscriber)),
      handler,
    }
  }
}

#[napi]
impl Subscriber {
  /// The key expression this subscriber listens on.
  #[napi(getter)]
  pub fn key_expr(&self) -> String {
    self.key_expr.clone()
  }

  /// Pull one sample, or `null` once the subscription is closed/undeclared.
  #[napi]
  pub async fn receive(&self) -> Result<Option<Sample>> {
    let handler = self.handler.clone();
    Ok(handler.recv().await)
  }

  /// Undeclare the subscription and release its resources.
  #[napi]
  pub async fn undeclare(&self) -> Result<()> {
    let taken = self.subscriber.lock().unwrap().take();
    if let Some(s) = taken {
      s.undeclare()
        .await
        .map_err(|e| zerr("subscriber.undeclare", e))?;
    }
    Ok(())
  }
}

/// `next()` returns a `'static` future, so it cannot borrow `self` — we clone the
/// cheap receiver and move it in.
#[napi]
impl AsyncGenerator for Subscriber {
  type Yield = Sample;
  type Next = ();
  type Return = ();

  fn next(
    &mut self,
    _value: Option<Self::Next>,
  ) -> impl Future<Output = Result<Option<Self::Yield>>> + Send + 'static {
    let handler = self.handler.clone();
    // `recv` yields `None` when all senders are dropped, ending iteration.
    async move { Ok(handler.recv().await) }
  }

  /// Called when the consumer `break`s out of `for await` (AsyncGenerator.return()):
  /// undeclare the subscription for clean teardown.
  fn complete(
    &mut self,
    _value: Option<Self::Return>,
  ) -> impl Future<Output = Result<Option<Self::Yield>>> + Send + 'static {
    let subscriber = self.subscriber.lock().unwrap().take();
    async move {
      if let Some(s) = subscriber {
        let _ = s.undeclare().await;
      }
      Ok(None)
    }
  }
}
