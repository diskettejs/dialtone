use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::{Wait, sample as zsample};

use crate::{
  bytes::*, config::*, handlers::*, key_expr::*, liveliness::*, macros::*, matching::*, miss::*,
  options::*, qos::*, sample::*, utils::*,
};

option_wrapper!(zenoh_ext::AdvancedPublisher<'static> as Publisher, "Undeclared publisher");

#[napi]
impl Publisher {
  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.get_ref()?.key_expr().clone().into())
  }

  #[napi(getter)]
  pub fn id(&self) -> napi::Result<EntityGlobalId> {
    Ok(self.get_ref()?.id().into())
  }

  #[napi(getter)]
  pub fn encoding(&self) -> napi::Result<Encoding> {
    Ok(self.get_ref()?.encoding().clone().into())
  }

  #[napi(getter)]
  pub fn congestion_control(&self) -> napi::Result<CongestionControl> {
    Ok(self.get_ref()?.congestion_control().into())
  }

  #[napi(getter)]
  pub fn priority(&self) -> napi::Result<Priority> {
    Ok(self.get_ref()?.priority().into())
  }

  #[napi]
  pub async fn put(
    &self,
    payload: Payload,
    options: Option<PublisherPutOptions>,
  ) -> napi::Result<()> {
    let payload = payload.into_zbytes();
    let PublisherPutOptions {
      encoding,
      timestamp,
      attachment,
    } = options.unwrap_or_default();
    let publisher = self.get_ref()?;

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
    let publisher = self.get_ref()?;

    build!(publisher.delete(), timestamp, attachment)
      .await
      .map_napi_err()
  }

  #[napi]
  pub async fn matching_status(&self) -> napi::Result<MatchingStatus> {
    let m = self.get_ref()?.matching_status().await.map_napi_err()?;
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
      .get_ref()?
      .matching_listener()
      .with(handler)
      .await
      .map_napi_err()?;

    Ok(listener.into())
  }

  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    Wait::wait(self.take()?.undeclare()).map_napi_err()
  }
}

option_wrapper!(
  zenoh_ext::AdvancedSubscriber<HandlerImpl<zsample::Sample>> as Subscriber,
  "Undeclared subscriber"
);

#[napi]
impl Subscriber {
  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.get_ref()?.key_expr().clone().into())
  }

  #[napi(getter)]
  pub fn id(&self) -> napi::Result<EntityGlobalId> {
    Ok(self.get_ref()?.id().into())
  }

  #[napi]
  pub async fn sample_miss_listener(
    &self,
    options: Option<SampleMissListenerOptions>,
  ) -> napi::Result<SampleMissListener> {
    let SampleMissListenerOptions { channel } = options.unwrap_or_default();
    let handler = into_handler(channel);

    let sample_listener = self
      .get_ref()?
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

    let subscriber = build!(self.get_ref()?.detect_publishers().with(handler), history)
      .await
      .map_napi_err()?;

    Ok(subscriber.into())
  }

  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    Wait::wait(self.take()?.undeclare()).map_napi_err()
  }
}

recv_handler!(Subscriber => Sample);
async_stream!(Subscriber => SampleStream yields Sample from zsample::Sample);
