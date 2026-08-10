use derive_more::From;
use napi_derive::napi;
use zenoh as z;

use crate::{config::*, key_expr::*, options::*, query::Replies, sample::Sample, utils::*};

#[napi]
#[derive(From)]
pub struct Liveliness(z::Session);

#[napi]
impl Liveliness {
  #[napi]
  pub async fn declare_token(&self, key_expr: KeyExprArg<'_>) -> napi::Result<LivelinessToken> {
    let expr = KeyExpr::try_from(key_expr)?;
    let token = self
      .0
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
    let LivelinessSubscriberOptions {
      history,
      channel_capacity,
    } = options.unwrap_or_default();
    let handler = fifo(channel_capacity);

    let subscriber = self
      .0
      .liveliness()
      .declare_subscriber(expr)
      .with(handler)
      .history(history.unwrap_or(false))
      .await
      .map_napi_err()?;

    Ok(subscriber.into())
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
      // cancellation_token,
      channel_capacity,
    } = options.unwrap_or_default();

    let timeout = duration_ms(timeout)?;
    let handler = fifo(channel_capacity);
    let session = self.0.clone();
    let mut builder = session.liveliness().get(expr).with(handler);

    if let Some(timeout) = timeout {
      builder = builder.timeout(timeout)
    }

    let rec = builder.await.map_napi_err()?;

    Ok(rec.into())
  }
}

#[napi]
#[derive(From)]
#[from(forward)]
pub struct LivelinessToken(Declared<z::liveliness::LivelinessToken>);

#[napi]
impl LivelinessToken {
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    z::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }
}

#[napi]
#[derive(From)]
#[from(forward)]
pub struct LivelinessSubscriber(
  Declared<z::pubsub::Subscriber<z::handlers::FifoChannelHandler<z::sample::Sample>>>,
);

#[napi]
impl LivelinessSubscriber {
  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.0.get()?.key_expr().clone().into())
  }

  #[napi(getter)]
  pub fn id(&self) -> napi::Result<EntityGlobalId> {
    Ok(self.0.get()?.id().into())
  }

  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    z::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
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
