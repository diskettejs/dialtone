use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::handlers::IntoHandler;
use zenoh::{Wait, liveliness as zliveliness, pubsub as zpubsub, sample as zsample};

use crate::{
  channels::*, config::*, error::*, handlers::HandlerImpl, key_expr::*, macros::*, options::*,
  sample::*,
};

wrapper!(zenoh::Session as Liveliness);

#[napi]
impl Liveliness {
  #[napi]
  pub async fn declare_token(&self, key_expr: KeyExprArg<'_>) -> napi::Result<LivelinessToken> {
    let expr = KeyExpr::try_from(key_expr)?;
    let token = self
      .inner
      .liveliness()
      .declare_token(expr)
      .await
      .map_napi_err()?;

    Ok(token.into())
  }

  #[napi]
  pub async fn declare_subscriber(
    &self,
    key_expr: KeyExprArg<'_>,
    options: Option<LivelinessSubscriberOptions>,
  ) -> napi::Result<LivelinessSubscriber> {
    let expr = KeyExpr::try_from(key_expr)?;
    let LivelinessSubscriberOptions { history, capacity } = options.unwrap_or_default();
    let (cb, _receiver) = FifoChannel::with_capacity(capacity).into_handler();

    let _subscriber = self
      .inner
      .liveliness()
      .declare_subscriber(expr)
      .with((cb, ()))
      .history(history.unwrap_or(false))
      .await
      .map_napi_err()?;

    // Ok(LivelinessSubscriber::new(subscriber, receiver))
    todo!("WIP migration to new generic channel system")
  }

  #[napi]
  pub async fn get(
    &self,
    key_expr: KeyExprArg<'_>,
    options: Option<LivelinessGetOptions>,
  ) -> napi::Result<()> {
    let expr = KeyExpr::try_from(key_expr)?;
    let LivelinessGetOptions {
      timeout,
      cancellation_token,
      capacity,
    } = options.unwrap_or_default();

    let timeout = duration_ms(timeout)?;
    let (cb, receiver) = FifoChannel::with_capacity(capacity).into_handler();
    let session = self.inner.clone();

    build!(
      session.liveliness().get(expr).with(cb),
      timeout,
      cancellation_token,
    )
    .await
    .map_napi_err()?;

    // Ok(receiver.into())
    todo!("migration to new channel handlers")
  }
}

option_wrapper!(zliveliness::LivelinessToken, "Undeclared liveliness token");

#[napi]
impl LivelinessToken {
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    Wait::wait(self.take()?.undeclare()).map_napi_err()
  }
}

option_wrapper!(
  zpubsub::Subscriber<HandlerImpl<zsample::Sample>> as LivelinessSubscriber,
  "Undeclared liveliness subscriber"
);

#[napi]
impl LivelinessSubscriber {
  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.get_ref()?.key_expr().clone().into())
  }

  #[napi(getter)]
  pub fn id(&self) -> napi::Result<EntityGlobalId> {
    Ok(self.get_ref()?.id().into())
  }

  #[napi(getter)]
  pub fn handler(&self) -> napi::Result<()> {
    todo!()
  }

  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    Wait::wait(self.take()?.undeclare()).map_napi_err()
  }
}
