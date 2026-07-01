use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{
  Wait, bytes as zbytes, cancellation as zcancellation, handlers::IntoHandler,
  key_expr as zkey_expr, qos as zqos, query as zquery, sample as zsample,
  session as zsession,
};

use crate::{
  bytes::*, channels::FifoChannel, config::*, error::*, handlers::Replies, key_expr::*,
  matching::*, options::*, qos::*, query::*,
};

#[napi]
pub struct Querier {
  id: zsession::EntityGlobalId,
  key_expr: zkey_expr::KeyExpr<'static>,
  congestion_control: zqos::CongestionControl,
  priority: zqos::Priority,
  accept_replies: zquery::ReplyKeyExpr,
  inner: Option<zquery::Querier<'static>>,
}

impl From<zquery::Querier<'static>> for Querier {
  fn from(querier: zquery::Querier<'static>) -> Self {
    Self {
      id: querier.id(),
      key_expr: querier.key_expr().clone(),
      congestion_control: querier.congestion_control(),
      priority: querier.priority(),
      accept_replies: querier.accept_replies(),
      inner: Some(querier),
    }
  }
}

#[napi]
impl Querier {
  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    self.key_expr.clone().into()
  }
  #[napi(getter)]
  pub fn id(&self) -> EntityGlobalId {
    self.id.into()
  }

  #[napi(getter)]
  pub fn congestion_control(&self) -> CongestionControl {
    self.congestion_control.into()
  }

  #[napi(getter)]
  pub fn priority(&self) -> Priority {
    self.priority.into()
  }

  #[napi(getter)]
  pub fn accept_replies(&self) -> ReplyKeyExpr {
    self.accept_replies.into()
  }

  #[napi]
  pub fn get<'env>(
    &self,
    env: &'env Env,
    options: Option<QuerierGetOptions>,
  ) -> napi::Result<PromiseRaw<'env, Replies>> {
    let querier = self
      .inner
      .as_ref()
      .ok_or_else(|| napi::Error::from_reason("querier has already been undeclared"))?;

    let QuerierGetOptions {
      parameters,
      payload,
      encoding,
      attachment,
      source_info,
      cancellation_token,
      capacity,
    } = options.unwrap_or_default();

    let parameters = parameters.map(|p| zquery::Parameters::from(p.as_ref()));
    let payload = payload.map(IntoZBytes::into_zbytes);
    let encoding = encoding.map(zbytes::Encoding::from);
    let attachment = attachment.map(IntoZBytes::into_zbytes);
    let source_info = source_info.map(|si| zsample::SourceInfo::from(si.as_ref()));
    let cancellation_token =
      cancellation_token.map(|ct| zcancellation::CancellationToken::from(&*ct));

    let (cb, receiver) = FifoChannel::with_capacity(capacity).into_handler();

    let mut builder = querier.get().with(cb);

    if let Some(parameters) = parameters {
      builder = builder.parameters(parameters);
    }

    if let Some(payload) = payload {
      builder = builder.payload(payload);
    }

    if let Some(encoding) = encoding {
      builder = builder.encoding(encoding);
    }

    if let Some(attachment) = attachment {
      builder = builder.attachment(attachment);
    }

    if let Some(source_info) = source_info {
      builder = builder.source_info(source_info);
    }

    if let Some(cancellation_token) = cancellation_token {
      builder = builder.cancellation_token(cancellation_token);
    }

    builder.wait().map_napi_err()?;

    env.spawn_future(async move { Ok(receiver.into()) })
  }

  #[napi]
  pub async fn matching_status(&self) -> napi::Result<MatchingStatus> {
    let querier = self
      .inner
      .as_ref()
      .ok_or_else(|| napi::Error::from_reason("querier has already been undeclared"))?;

    let status = querier.matching_status().await.map_napi_err()?;

    Ok(status.into())
  }

  #[napi]
  pub fn matching_listener<'env>(
    &self,
    env: &'env Env,
    options: Option<MatchingListenerOptions>,
  ) -> napi::Result<PromiseRaw<'env, MatchingListener>> {
    let querier = self
      .inner
      .as_ref()
      .ok_or_else(|| napi::Error::from_reason("querier has already been undeclared"))?;

    let MatchingListenerOptions { capacity } = options.unwrap_or_default();
    let (cb, receiver) = FifoChannel::with_capacity(capacity).into_handler();
    let listener = querier.matching_listener().with(cb).wait().map_napi_err()?;

    env.spawn_future(async move { Ok(MatchingListener::new(listener, receiver)) })
  }

  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    let querier = self
      .inner
      .take()
      .ok_or_else(|| napi::Error::from_reason("querier has already been undeclared"))?;

    env.spawn_future(async move { querier.undeclare().await.map_napi_err() })
  }
}
