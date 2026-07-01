use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{handlers as zhandlers, matching as zmatching};

use crate::{
  error::*,
  macros::{channel_forward, wrapper},
};

wrapper!(zmatching::MatchingStatus);

#[napi]
impl MatchingStatus {
  #[napi(getter)]
  pub fn matching(&self) -> bool {
    self.inner.matching()
  }
}

#[napi]
pub struct MatchingListener {
  inner: Option<zmatching::MatchingListener<()>>,
  receiver: crate::handlers::FifoChannelHandler<zmatching::MatchingStatus>,
}

impl MatchingListener {
  pub(crate) fn new(
    inner: zmatching::MatchingListener<()>,
    receiver: zhandlers::FifoChannelHandler<zmatching::MatchingStatus>,
  ) -> Self {
    Self {
      inner: Some(inner),
      receiver: receiver.into(),
    }
  }
}

#[napi]
impl MatchingListener {
  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    let listener = self
      .inner
      .take()
      .ok_or_else(|| napi::Error::from_reason("matching listener has already been undeclared"))?;

    env.spawn_future(async move { listener.undeclare().await.map_napi_err() })
  }
}

channel_forward!(MatchingListener, receiver, MatchingStatus);
