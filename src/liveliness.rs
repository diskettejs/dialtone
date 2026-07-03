use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::{Wait, liveliness as zliveliness, pubsub as zpubsub, sample as zsample};

use crate::{config::*, handlers::*, key_expr::*, macros::*, options::*, sample::*, utils::*};

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
    let LivelinessSubscriberOptions { history, channel } = options.unwrap_or_default();
    let handler = into_handler(channel);

    let subscriber = self
      .inner
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
  ) -> napi::Result<ReplyHandler> {
    let expr = KeyExpr::try_from(key_expr)?;
    let LivelinessGetOptions {
      timeout,
      cancellation_token,
      channel,
    } = options.unwrap_or_default();

    let timeout = duration_ms(timeout)?;
    let handler = into_handler(channel);
    let session = self.inner.clone();

    let rec = build!(
      session.liveliness().get(expr).with(handler),
      timeout,
      cancellation_token,
    )
    .await
    .map_napi_err()?;

    Ok(rec.into())
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

  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    Wait::wait(self.take()?.undeclare()).map_napi_err()
  }
}

recv_handler!(LivelinessSubscriber => Sample);
async_stream!(LivelinessSubscriber => LivelinessSampleStream yields Sample from zsample::Sample);
