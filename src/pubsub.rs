use derive_more::From;
use napi::bindgen_prelude::AsyncGenerator;
use napi_derive::napi;

use crate::{
  bytes::{BytesBuffer, Encoding},
  config::EntityGlobalId,
  key_expr::KeyExpr,
  liveliness::LivelinessSubscriber,
  matching::{MatchingListener, MatchingStatus},
  miss::SampleMissListener,
  options::{
    LivelinessSubscriberOptions, MatchingListenerOptions, PublisherDeleteOptions,
    PublisherPutOptions, SampleMissListenerOptions,
  },
  qos::{CongestionControl, Priority},
  sample::Sample,
  utils::{Declared, MapNapiErr, fifo},
};

/// A publisher declared on a key expression, used to send data repeatedly without
/// re-resolving the key expression on each publication.
///
/// On top of a plain publication, a publisher can keep a {@link PublisherOptions.cache}
/// of the last samples so that subscribers can retrieve them as history or ask for their
/// retransmission, announce a sequence number so subscribers can detect misses through
/// {@link PublisherOptions.sampleMissDetection}, and make itself discoverable through
/// {@link PublisherOptions.publisherDetection}.
///
/// {@link Publisher.undeclare} consumes the publisher; every member throws once it has
/// been undeclared.
#[napi]
#[derive(From)]
#[from(forward)]
pub struct Publisher(Declared<zenoh_ext::AdvancedPublisher<'static>>);

#[napi]
impl Publisher {
  /// The key expression this publisher writes to.
  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.0.get()?.key_expr().clone().into())
  }

  /// The global identifier of this publisher.
  #[napi(getter)]
  pub fn id(&self) -> napi::Result<EntityGlobalId> {
    Ok(self.0.get()?.id().into())
  }

  /// The encoding used when publishing data.
  ///
  /// A single publication can override it with {@link PublisherPutOptions.encoding}.
  #[napi(getter)]
  pub fn encoding(&self) -> napi::Result<Encoding> {
    Ok(self.0.get()?.encoding().clone().into())
  }

  /// The congestion control applied when routing the published data.
  #[napi(getter)]
  pub fn congestion_control(&self) -> napi::Result<CongestionControl> {
    Ok(self.0.get()?.congestion_control().into())
  }

  /// The priority applied when routing the published data.
  #[napi(getter)]
  pub fn priority(&self) -> napi::Result<Priority> {
    Ok(self.0.get()?.priority().into())
  }

  /// Publishes a payload on this publisher's key expression.
  ///
  /// The matching subscribers receive a sample whose kind is `Put`.
  #[napi]
  pub async fn put(
    &self,
    payload: BytesBuffer,
    options: Option<PublisherPutOptions>,
  ) -> napi::Result<()> {
    let PublisherPutOptions {
      encoding,
      attachment,
    } = options.unwrap_or_default();
    let publisher = self.0.get()?;
    let mut builder = publisher.put(payload);

    if let Some(encoding) = encoding {
      builder = builder.encoding(encoding);
    }

    if let Some(attachment) = attachment {
      builder = builder.attachment(attachment);
    }

    builder.await.map_napi_err()
  }

  /// Declares that the data associated with this publisher's key expression is deleted.
  ///
  /// The matching subscribers receive a sample whose kind is `Delete`.
  #[napi]
  pub async fn delete(&self, options: Option<PublisherDeleteOptions>) -> napi::Result<()> {
    let PublisherDeleteOptions { attachment } = options.unwrap_or_default();
    let publisher = self.0.get()?;
    let mut builder = publisher.delete();

    if let Some(attachment) = attachment {
      builder = builder.attachment(attachment);
    }

    builder.await.map_napi_err()
  }

  /// Reads the current matching status of this publisher.
  ///
  /// @returns A {@link `MatchingStatus`} whose {@link MatchingStatus.matching} is `true`
  /// if there exist subscribers matching this publisher's key expression.
  #[napi]
  pub async fn matching_status(&self) -> napi::Result<MatchingStatus> {
    let m = self.0.get()?.matching_status().await.map_napi_err()?;
    Ok(m.into())
  }

  /// Declares a listener notified each time the matching status of this publisher
  /// changes, i.e. each time it gains its first matching subscriber or loses its last
  /// one.
  #[napi]
  pub async fn matching_listener(
    &self,
    options: Option<MatchingListenerOptions>,
  ) -> napi::Result<MatchingListener> {
    let MatchingListenerOptions { channel_capacity } = options.unwrap_or_default();
    let handler = fifo(channel_capacity);

    let listener = self
      .0
      .get()?
      .matching_listener()
      .with(handler)
      .await
      .map_napi_err()?;

    Ok(listener.into())
  }

  /// Undeclares this publisher, informing the network that it need not optimize
  /// publications for its key expression anymore.
  ///
  /// @throws If this publisher has already been undeclared.
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    zenoh::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }
}

/// A subscriber receiving the samples published on the key expressions matching its own.
///
/// On top of a plain subscription, a subscriber can query the matching publishers for
/// {@link SubscriberOptions.history}, detect the samples it missed and ask for their
/// {@link SubscriberOptions.recovery}, and make itself discoverable through
/// {@link SubscriberOptions.subscriberDetection}. The counterpart features must be
/// enabled on the publisher side.
///
/// {@link Subscriber.undeclare} consumes the subscriber; every member throws once it has
/// been undeclared.
#[napi]
#[derive(From)]
#[from(forward)]
pub struct Subscriber(
  Declared<
    zenoh_ext::AdvancedSubscriber<zenoh::handlers::FifoChannelHandler<zenoh::sample::Sample>>,
  >,
);

#[napi]
impl Subscriber {
  /// The key expression this subscriber subscribes to.
  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.0.get()?.key_expr().clone().into())
  }

  /// The global identifier of this subscriber.
  #[napi(getter)]
  pub fn id(&self) -> napi::Result<EntityGlobalId> {
    Ok(self.0.get()?.id().into())
  }

  /// Declares a listener reporting the samples this subscriber missed.
  ///
  /// Missed samples can only be detected from publishers that enable
  /// {@link PublisherOptions.sampleMissDetection}.
  #[napi]
  pub async fn sample_miss_listener(
    &self,
    options: Option<SampleMissListenerOptions>,
  ) -> napi::Result<SampleMissListener> {
    let SampleMissListenerOptions { channel_capacity } = options.unwrap_or_default();
    let handler = fifo(channel_capacity);

    let sample_listener = self
      .0
      .get()?
      .sample_miss_listener()
      .with(handler)
      .await
      .map_napi_err()?;

    Ok(sample_listener.into())
  }

  /// Declares a liveliness subscriber reporting the publishers matching this subscriber
  /// as they appear and disappear.
  ///
  /// Only publishers that enable {@link PublisherOptions.publisherDetection} can be
  /// detected.
  #[napi]
  pub async fn detect_publishers(
    &self,
    options: Option<LivelinessSubscriberOptions>,
  ) -> napi::Result<LivelinessSubscriber> {
    let LivelinessSubscriberOptions {
      history,
      channel_capacity,
    } = options.unwrap_or_default();
    let handler = fifo(channel_capacity);
    let mut builder = self.0.get()?.detect_publishers().with(handler);

    if let Some(history) = history {
      builder = builder.history(history);
    }

    let subscriber = builder.await.map_napi_err()?;

    Ok(subscriber.into())
  }

  /// Undeclares this subscriber, so that no further sample is delivered to it.
  ///
  /// @throws If this subscriber has already been undeclared.
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    zenoh::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }

  /// Iterates over the samples delivered to this subscriber.
  ///
  /// @returns A {@link `SampleIter`} that yields each buffered sample and completes once
  /// the subscriber stops receiving, e.g. after it is undeclared or the session is
  /// closed.
  /// @throws If this subscriber has already been undeclared.
  #[napi]
  pub fn receive(&self) -> napi::Result<SampleIter> {
    let handler = self.0.get()?.handler().clone();

    Ok(handler.into())
  }
}

/// A stream of the samples delivered to a subscriber.
#[napi(async_iterator)]
#[derive(From)]
pub struct SampleIter(zenoh::handlers::FifoChannelHandler<zenoh::sample::Sample>);

#[napi]
impl AsyncGenerator for SampleIter {
  type Yield = Sample;
  type Next = ();
  type Return = ();

  fn next(
    &mut self,
    _value: Option<Self::Next>,
  ) -> impl Future<Output = napi::Result<Option<Self::Yield>>> + Send + 'static {
    let receiver = self.0.clone();

    async move {
      match receiver.recv_async().await {
        Ok(sample) => Ok(Some(sample.into())),
        Err(_) => Ok(None),
      }
    }
  }
}
