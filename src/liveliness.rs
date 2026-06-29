use std::time::Duration;

use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::handlers::IntoHandler;
use zenoh::{
  cancellation as zcancellation, handlers as zhandlers, key_expr as zkey_expr,
  liveliness as zliveliness, pubsub as zpubsub, sample as zsample,
  session as zsession,
};

use crate::{
  channels::*, config::*, error::*, handlers::Replies, key_expr::*, options::*, sample::Sample,
};

#[napi]
pub struct Liveliness {
  inner: zenoh::Session,
}

impl From<zenoh::Session> for Liveliness {
  fn from(session: zenoh::Session) -> Self {
    Self { inner: session }
  }
}

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
  pub fn get<'env>(
    &self,
    env: &'env Env,
    key_expr: KeyExprArg,
    options: Option<LivelinessGetOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, Replies>> {
    let expr = KeyExpr::try_from(key_expr)?;
    let LivelinessGetOptions {
      timeout,
      cancellation_token,
      capacity,
    } = options.unwrap_or_default();

    let timeout = timeout
      .map(|ms| Duration::try_from_secs_f64(ms / 1000.0).map_napi_err())
      .transpose()?;
    let cancellation_token =
      cancellation_token.map(|ct| zcancellation::CancellationToken::from(&*ct));
    let (cb, receiver) = FifoChannel::with_capacity(capacity).into_handler();
    let session = self.inner.clone();

    env.spawn_future(async move {
      let mut builder = session.liveliness().get(expr).with(cb);

      if let Some(timeout) = timeout {
        builder = builder.timeout(timeout);
      }

      if let Some(cancellation_token) = cancellation_token {
        builder = builder.cancellation_token(cancellation_token);
      }

      builder.await.map_napi_err()?;
      Ok(receiver.into())
    })
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
  pub async fn recv(&self) -> napi::Result<Sample> {
    self.receiver.recv::<Sample>().await
  }

  #[napi]
  pub fn try_recv(&self) -> napi::Result<Option<Sample>> {
    self.receiver.try_recv::<Sample>()
  }

  #[napi]
  pub fn drain(&self) -> Vec<Sample> {
    self.receiver.drain::<Sample>()
  }

  #[napi]
  pub fn is_disconnected(&self) -> bool {
    self.receiver.is_disconnected()
  }

  #[napi]
  pub fn is_empty(&self) -> bool {
    self.receiver.is_empty()
  }

  #[napi]
  pub fn is_full(&self) -> bool {
    self.receiver.is_full()
  }

  #[napi]
  pub fn len(&self) -> u32 {
    self.receiver.len()
  }

  #[napi]
  pub fn capacity(&self) -> Option<u32> {
    self.receiver.capacity()
  }

  #[napi]
  pub fn sender_count(&self) -> u32 {
    self.receiver.sender_count()
  }

  #[napi]
  pub fn receiver_count(&self) -> u32 {
    self.receiver.receiver_count()
  }

  #[napi]
  pub fn stream<'env>(&self, env: &'env Env) -> napi::Result<ReadableStream<'env, Sample>> {
    self.receiver.stream::<Sample>(env)
  }

  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    let subscriber = self.inner.take().ok_or_else(|| {
      napi::Error::from_reason("liveliness subscriber has already been undeclared")
    })?;

    env.spawn_future(async move { subscriber.undeclare().await.map_napi_err() })
  }
}
