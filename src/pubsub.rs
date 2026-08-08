use derive_more::From;
use napi_derive::napi;

use crate::{
  bytes::*, config::*, handlers::*, key_expr::*, liveliness::*, matching::*, miss::*, options::*,
  qos::*, utils::*,
};

#[derive(From)]
#[from(forward)]
#[napi]
pub struct Publisher(Declared<Publisher, zenoh_ext::AdvancedPublisher<'static>>);

#[napi]
impl Publisher {
  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.0.get()?.key_expr().clone().into())
  }

  #[napi(getter)]
  pub fn id(&self) -> napi::Result<EntityGlobalId> {
    Ok(self.0.get()?.id().into())
  }

  #[napi(getter)]
  pub fn encoding(&self) -> napi::Result<Encoding> {
    Ok(self.0.get()?.encoding().clone().into())
  }

  #[napi(getter)]
  pub fn congestion_control(&self) -> napi::Result<CongestionControl> {
    Ok(self.0.get()?.congestion_control().into())
  }

  #[napi(getter)]
  pub fn priority(&self) -> napi::Result<Priority> {
    Ok(self.0.get()?.priority().into())
  }

  #[napi]
  pub async fn put(
    &self,
    payload: BytesLike,
    options: Option<PublisherPutOptions>,
  ) -> napi::Result<()> {
    let payload = payload.into_zbytes();
    let PublisherPutOptions {
      encoding,
      timestamp,
      attachment,
    } = options.unwrap_or_default();
    let publisher = self.0.get()?;

    build!(publisher.put(payload), encoding, timestamp, attachment)
      .await
      .map_napi_err()
  }

  #[napi]
  pub async fn delete(&self, options: Option<PublisherDeleteOptions>) -> napi::Result<()> {
    let PublisherDeleteOptions {
      timestamp,
      attachment,
    } = options.unwrap_or_default();
    let publisher = self.0.get()?;

    build!(publisher.delete(), timestamp, attachment)
      .await
      .map_napi_err()
  }

  #[napi]
  pub async fn matching_status(&self) -> napi::Result<MatchingStatus> {
    let m = self.0.get()?.matching_status().await.map_napi_err()?;
    Ok(m.into())
  }

  #[napi]
  pub async fn matching_listener(
    &self,
    options: Option<MatchingListenerOptions>,
  ) -> napi::Result<MatchingListener> {
    let MatchingListenerOptions { channel } = options.unwrap_or_default();
    let handler = into_handler(channel);

    let listener = self
      .0
      .get()?
      .matching_listener()
      .with(handler)
      .await
      .map_napi_err()?;

    Ok(listener.into())
  }

  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    zenoh::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }
}

#[derive(From)]
#[from(forward)]
#[napi]
pub struct Subscriber(
  Declared<Subscriber, zenoh_ext::AdvancedSubscriber<HandlerImpl<zenoh::sample::Sample>>>,
);

#[napi]
impl Subscriber {
  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.0.get()?.key_expr().clone().into())
  }

  #[napi(getter)]
  pub fn id(&self) -> napi::Result<EntityGlobalId> {
    Ok(self.0.get()?.id().into())
  }

  #[napi]
  pub async fn sample_miss_listener(
    &self,
    options: Option<SampleMissListenerOptions>,
  ) -> napi::Result<SampleMissListener> {
    let SampleMissListenerOptions { channel } = options.unwrap_or_default();
    let handler = into_handler(channel);

    let sample_listener = self
      .0
      .get()?
      .sample_miss_listener()
      .with(handler)
      .await
      .map_napi_err()?;

    Ok(sample_listener.into())
  }

  #[napi]
  pub async fn detect_publishers(
    &self,
    options: Option<LivelinessSubscriberOptions>,
  ) -> napi::Result<LivelinessSubscriber> {
    let LivelinessSubscriberOptions { history, channel } = options.unwrap_or_default();
    let handler = into_handler(channel);

    let subscriber = build!(self.0.get()?.detect_publishers().with(handler), history)
      .await
      .map_napi_err()?;

    Ok(subscriber.into())
  }

  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    zenoh::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
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
