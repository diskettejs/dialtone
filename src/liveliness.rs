use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::handlers::IntoHandler;
use zenoh::{
  handlers as zhandlers, key_expr as zkey_expr, liveliness as zliveliness, pubsub as zpubsub,
  sample as zsample, session as zsession,
};

use crate::{
  channels::*,
  config::*,
  error::*,
  handlers::Replies,
  key_expr::*,
  macros::{build, channel_forward, wrapper},
  options::*,
  sample::Sample,
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
    let (cb, receiver) = FifoChannel::with_capacity(capacity).into_handler();

    let subscriber = self
      .inner
      .liveliness()
      .declare_subscriber(expr)
      .with((cb, ()))
      .history(history.unwrap_or(false))
      .await
      .map_napi_err()?;

    Ok(LivelinessSubscriber::new(subscriber, receiver))
  }

  #[napi]
  pub async fn get(
    &self,
    key_expr: KeyExprArg<'_>,
    options: Option<LivelinessGetOptions>,
  ) -> napi::Result<Replies> {
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

    Ok(receiver.into())
  }
}

#[napi]
pub struct LivelinessToken {
  inner: Option<zliveliness::LivelinessToken>,
}

impl From<zliveliness::LivelinessToken> for LivelinessToken {
  fn from(inner: zliveliness::LivelinessToken) -> Self {
    Self { inner: Some(inner) }
  }
}

#[napi]
impl LivelinessToken {
  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    let token = self
      .inner
      .take()
      .ok_or_else(|| napi::Error::from_reason("liveliness token has already been undeclared"))?;

    env.spawn_future(async move { token.undeclare().await.map_napi_err() })
  }
}

#[napi]
pub struct LivelinessSubscriber {
  id: zsession::EntityGlobalId,
  key_expr: zkey_expr::KeyExpr<'static>,
  inner: Option<zpubsub::Subscriber<()>>,
  receiver: crate::handlers::FifoChannelHandler<zsample::Sample>,
}

impl LivelinessSubscriber {
  pub fn new(
    inner: zpubsub::Subscriber<()>,
    receiver: zhandlers::FifoChannelHandler<zsample::Sample>,
  ) -> Self {
    Self {
      id: inner.id(),
      key_expr: inner.key_expr().clone(),
      inner: Some(inner),
      receiver: receiver.into(),
    }
  }
}

#[napi]
impl LivelinessSubscriber {
  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    self.key_expr.clone().into()
  }

  #[napi(getter)]
  pub fn id(&self) -> EntityGlobalId {
    self.id.into()
  }

  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    let subscriber = self.inner.take().ok_or_else(|| {
      napi::Error::from_reason("liveliness subscriber has already been undeclared")
    })?;

    env.spawn_future(async move { subscriber.undeclare().await.map_napi_err() })
  }
}

channel_forward!(LivelinessSubscriber, receiver, Sample);
