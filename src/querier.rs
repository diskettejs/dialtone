use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{Wait, handlers::IntoHandler, query as zquery};

use crate::{
  channels::FifoChannel, config::*, error::*, key_expr::*, macros::*, matching::*, options::*,
  qos::*, query::*,
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
  pub async fn get(&self, options: Option<QuerierGetOptions>) -> napi::Result<()> {
    let querier = self.get_ref()?;

    let QuerierGetOptions {
      parameters,
      payload,
      encoding,
      attachment,
      source_info,
      cancellation_token,
    } = options.unwrap_or_default();

    // NOTE: temp hardcoded because of ongoing channel handlers rework
    let (cb, receiver) = FifoChannel::new(256).into_handler();

    build!(
      querier.get().with(cb),
      parameters,
      payload,
      encoding,
      attachment,
      source_info,
      cancellation_token,
    )
    .await
    .map_napi_err()?;

    // Ok(receiver.into())
    todo!("migration to new channel handlers")
  }

  #[napi]
  pub async fn matching_status(&self) -> napi::Result<MatchingStatus> {
    let status = self.get_ref()?.matching_status().await.map_napi_err()?;

    Ok(status.into())
  }

  #[napi]
  pub fn matching_listener<'env>(
    &self,
    env: &'env Env,
    options: Option<MatchingListenerOptions>,
  ) -> napi::Result<PromiseRaw<'env, MatchingListener>> {
    let querier = self.get_ref()?;

    let MatchingListenerOptions {} = options.unwrap_or_default();
    // NOTE: temp hardcoded because of ongoing channel handlers rework
    let (cb, _receiver) = FifoChannel::new(256).into_handler();
    let _listener = querier.matching_listener().with(cb).wait().map_napi_err()?;

    todo!("WIP migration to new generic channel system")
  }

  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    Wait::wait(self.take()?.undeclare()).map_napi_err()
  }
}
