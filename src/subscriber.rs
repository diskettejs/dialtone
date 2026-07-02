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
    let SampleMissListenerOptions { capacity } = options.unwrap_or_default();
    let (cb, _receiver) = FifoChannel::with_capacity(capacity).into_handler();

    let _sample_listener = self
      .get_ref()?
      .sample_miss_listener()
      .with((cb, ()))
      .await
      .map_napi_err()?;

    // Ok(SampleMissListener::new(sample_listener, receiver))
    todo!("WIP migration to new generic channel system")
  }

  #[napi(getter)]
  pub fn handler(&self) -> napi::Result<()> {
    // Ok(self.get_ref()?.handler())
    todo!()
  }

  #[napi]
  pub async fn detect_publishers(
    &self,
    options: Option<LivelinessSubscriberOptions>,
  ) -> napi::Result<LivelinessSubscriber> {
    let LivelinessSubscriberOptions { history, capacity } = options.unwrap_or_default();
    let (cb, _receiver) = FifoChannel::with_capacity(capacity).into_handler();

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
