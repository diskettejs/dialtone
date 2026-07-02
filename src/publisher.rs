use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::{Wait, handlers::IntoHandler};

use crate::{
  bytes::*, channels::FifoChannel, config::*, encoding::*, error::*, key_expr::*, macros::build,
  macros::option_wrapper, matching::*, options::*, qos::*,
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
    let MatchingListenerOptions {} = options.unwrap_or_default();

    // NOTE: temp hardcoded because of ongoing channel handlers rework
    let (cb, _receiver) = FifoChannel::new(256).into_handler();

    let _listener = self
      .get_ref()?
      .matching_listener()
      .with((cb, ()))
      .await
      .map_napi_err()?;

    // Ok(MatchingListener::new(listener, receiver))
    todo!("WIP migration to new generic channel system")
  }

  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    Wait::wait(self.take()?.undeclare()).map_napi_err()
  }
}
