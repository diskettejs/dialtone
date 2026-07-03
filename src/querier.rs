use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::{Wait, query as zquery};

use crate::{
  config::*,
  error::*,
  handlers::{ReplyHandler, into_handler},
  key_expr::*,
  macros::*,
  matching::*,
  options::*,
  qos::*,
  query::*,
};

option_wrapper!(zquery::Querier<'static>, "Undeclared querier");

#[napi]
impl Querier {
  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.get_ref()?.key_expr().clone().into())
  }

  #[napi(getter)]
  pub fn id(&self) -> napi::Result<EntityGlobalId> {
    Ok(self.get_ref()?.id().into())
  }

  #[napi(getter)]
  pub fn congestion_control(&self) -> napi::Result<CongestionControl> {
    Ok(self.get_ref()?.congestion_control().into())
  }

  #[napi(getter)]
  pub fn priority(&self) -> napi::Result<Priority> {
    Ok(self.get_ref()?.priority().into())
  }

  #[napi(getter)]
  pub fn accept_replies(&self) -> napi::Result<ReplyKeyExpr> {
    Ok(self.get_ref()?.accept_replies().into())
  }

  #[napi]
  pub async fn get(&self, options: Option<QuerierGetOptions>) -> napi::Result<ReplyHandler> {
    let QuerierGetOptions {
      parameters,
      payload,
      encoding,
      attachment,
      source_info,
      cancellation_token,
      channel,
    } = options.unwrap_or_default();

    let handler = into_handler(channel);

    let receiver = build!(
      self.get_ref()?.get().with(handler),
      parameters,
      payload,
      encoding,
      attachment,
      source_info,
      cancellation_token,
    )
    .await
    .map_napi_err()?;

    Ok(receiver.into())
  }

  #[napi]
  pub async fn matching_status(&self) -> napi::Result<MatchingStatus> {
    let status = self.get_ref()?.matching_status().await.map_napi_err()?;

    Ok(status.into())
  }

  #[napi]
  pub async fn matching_listener(
    &self,
    options: Option<MatchingListenerOptions>,
  ) -> napi::Result<MatchingListener> {
    let MatchingListenerOptions { channel } = options.unwrap_or_default();
    let handler = into_handler(channel);

    let listener = self
      .get_ref()?
      .matching_listener()
      .with(handler)
      .await
      .map_napi_err()?;

    Ok(listener.into())
  }

  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    Wait::wait(self.take()?.undeclare()).map_napi_err()
  }
}
