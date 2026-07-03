use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{
  Wait,
  handlers::{self as zhandlers, IntoHandler},
  sample as zsample,
};

use crate::{
  channels::*, config::*, error::*, handlers::HandlerImpl, key_expr::*, liveliness::*, macros::*,
  miss::*, options::*, sample::*,
};

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
    let SampleMissListenerOptions {} = options.unwrap_or_default();
    // NOTE: temp hardcoded because of ongoing channel handlers rework
    let (cb, _receiver) = FifoChannel::new(256).into_handler();

    let _sample_listener = self
      .get_ref()?
      .sample_miss_listener()
      .with((cb, ()))
      .await
      .map_napi_err()?;

    // Ok(SampleMissListener::new(sample_listener))
    todo!("WIP migration to new generic channel system")
  }

  #[napi]
  pub async fn recv(&self) -> napi::Result<Sample> {
    let sample = self.get_ref()?.handler().recv().await?;
    Ok(sample.into())
  }

  #[napi]
  pub fn try_recv(&self) -> napi::Result<Option<Sample>> {
    Ok(self.get_ref()?.handler().try_recv()?.map(Into::into))
  }

  #[napi]
  pub async fn detect_publishers(
    &self,
    options: Option<LivelinessSubscriberOptions>,
  ) -> napi::Result<LivelinessSubscriber> {
    let LivelinessSubscriberOptions { history } = options.unwrap_or_default();
    let (cb, _receiver) = FifoChannel::new(256).into_handler();

    let _subscriber = build!(self.get_ref()?.detect_publishers().with((cb, ())), history)
      .await
      .map_napi_err()?;

    // Ok(LivelinessSubscriber::new(subscriber, receiver))
    todo!("WIP migration to new generic channel system")
  }

  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    Wait::wait(self.take()?.undeclare()).map_napi_err()
  }
}
