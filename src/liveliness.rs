use std::time::Duration;

use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{
  cancellation as zcancellation, key_expr as zkey_expr, liveliness as zliveliness,
  pubsub as zpubsub, query as zquery, sample as zsample, session as zsession,
};

use crate::{config::*, error::*, key_expr::*, options::*};

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
    todo!()
    // let expr = KeyExpr::try_from(key_expr)?;
    // let session = self.inner.get()?;
    // let token = session
    //   .liveliness()
    //   .declare_token(expr)
    //   .await
    //   .map_napi_err()?;

    // Ok(token.into())
  }

  #[napi]
  pub fn declare_subscriber<'env>(
    &self,
    env: &'env Env,
    key_expr: KeyExprArg,
    options: Option<LivelinessSubscriberOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, LivelinessSubscriber>> {
    todo!()
    // let expr = KeyExpr::try_from(key_expr)?;
    // let LivelinessSubscriberOptions { channel, history } = options.unwrap_or_default();
    // let session = self.inner.get()?.clone();

    // env.spawn_future(async move {
    //   let inner = session
    //     .liveliness()
    //     .declare_subscriber(expr)
    //     .with(zenoh_channel)
    //     .history(history.unwrap_or(false))
    //     .await
    //     .map_napi_err()?;
    //   Ok(LivelinessSubscriber::new(inner, receiver))
    // })
  }

  #[napi]
  pub fn get<'env>(
    &self,
    env: &'env Env,
    key_expr: KeyExprArg,
    options: Option<LivelinessGetOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, Either<(), ()>>> {
    todo!()

    // let expr = KeyExpr::try_from(key_expr)?;
    // let LivelinessGetOptions {
    //   timeout,
    //   cancellation_token,
    //   channel,
    // } = options.unwrap_or_default();
    // let timeout = timeout
    //   .map(|ms| Duration::try_from_secs_f64(ms / 1000.0).map_napi_err())
    //   .transpose()?;
    // let cancellation_token =
    //   cancellation_token.map(|ct| zcancellation::CancellationToken::from(&*ct));
    // let session = self.inner.get()?.clone();

    // env.spawn_future(async move {
    //   let mut builder = session.liveliness().get(expr).with(callback);
    //   if let Some(timeout) = timeout {
    //     builder = builder.timeout(timeout);
    //   }
    //   if let Some(cancellation_token) = cancellation_token {
    //     builder = builder.cancellation_token(cancellation_token);
    //   }
    //   builder.await.map_napi_err()?;
    //   Ok(receiver.handler())
    // })
  }
}

#[napi]
pub struct LivelinessToken {
  inner: zliveliness::LivelinessToken,
}

impl From<zliveliness::LivelinessToken> for LivelinessToken {
  fn from(inner: zliveliness::LivelinessToken) -> Self {
    Self { inner }
  }
}

#[napi]
impl LivelinessToken {
  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    todo!()
  }
}

#[napi]
pub struct LivelinessSubscriber {
  inner: zpubsub::Subscriber<()>,
}

impl LivelinessSubscriber {
  pub(crate) fn new(inner: zpubsub::Subscriber<()>) -> Self {
    Self { inner }
  }
}

#[napi]
impl LivelinessSubscriber {
  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    self.inner.key_expr().clone().into()
  }

  #[napi(getter)]
  pub fn id(&self) -> EntityGlobalId {
    self.inner.id().into()
  }

  #[napi(getter)]
  pub fn handler(&self) -> Either<(), ()> {
    todo!()
  }

  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    todo!()
  }
}
