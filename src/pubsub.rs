use derive_more::From;
use napi_derive::napi;

use crate::{
  bytes::*, config::*, key_expr::*, liveliness::*, matching::*, miss::*, options::*, qos::*,
  sample::Sample, utils::*,
};

#[derive(From)]
#[from(forward)]
#[napi]
pub struct Publisher(Declared<zenoh_ext::AdvancedPublisher<'static>>);

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
      builder = builder.encoding(encoding)
    }

    if let Some(attachment) = attachment {
      builder = builder.attachment(attachment)
    }

    builder.await.map_napi_err()
  }

  #[napi]
  pub async fn delete(&self, options: Option<PublisherDeleteOptions>) -> napi::Result<()> {
    let PublisherDeleteOptions { attachment } = options.unwrap_or_default();
    let publisher = self.0.get()?;
    let mut builder = publisher.delete();

    if let Some(attachment) = attachment {
      builder = builder.attachment(attachment)
    }

    builder.await.map_napi_err()
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

  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    zenoh::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }
}

#[derive(From)]
#[from(forward)]
#[napi]
pub struct Subscriber(
  Declared<
    zenoh_ext::AdvancedSubscriber<zenoh::handlers::FifoChannelHandler<zenoh::sample::Sample>>,
  >,
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
      builder = builder.history(history)
    }

    let subscriber = builder.await.map_napi_err()?;

    Ok(subscriber.into())
  }

  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    zenoh::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }

  #[napi]
  pub async fn recv(&self) -> napi::Result<Sample> {
    let sample = self.0.get()?.recv_async().await.map_napi_err()?;

    Ok(sample.into())
  }

  #[napi]
  pub fn try_recv(&self) -> napi::Result<Option<Sample>> {
    Ok(self.0.get()?.try_recv().map_napi_err()?.map(Into::into))
  }
}
