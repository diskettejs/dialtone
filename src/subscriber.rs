use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{
  Wait,
  handlers::{self as zhandlers, IntoHandler},
  sample as zsample,
};

use crate::{
  config::*,
  error::*,
  handlers::{HandlerImpl, into_handler},
  key_expr::*,
  liveliness::*,
  macros::*,
  miss::*,
  options::*,
  sample::*,
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
