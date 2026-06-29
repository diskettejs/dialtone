use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{handlers as zhandlers, matching as zmatching};

use crate::error::*;

#[napi]
pub struct MatchingStatus {
  inner: zmatching::MatchingStatus,
}

impl From<zmatching::MatchingStatus> for MatchingStatus {
  fn from(inner: zmatching::MatchingStatus) -> Self {
    Self { inner }
  }
}

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
  pub async fn recv(&self) -> napi::Result<MatchingStatus> {
    self.receiver.recv::<MatchingStatus>().await
  }

  #[napi]
  pub fn try_recv(&self) -> napi::Result<Option<MatchingStatus>> {
    self.receiver.try_recv::<MatchingStatus>()
  }

  #[napi]
  pub fn drain(&self) -> Vec<MatchingStatus> {
    self.receiver.drain::<MatchingStatus>()
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
  pub fn stream<'env>(
    &self,
    env: &'env Env,
  ) -> napi::Result<ReadableStream<'env, MatchingStatus>> {
    self.receiver.stream::<MatchingStatus>(env)
  }

  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    let listener = self
      .inner
      .take()
      .ok_or_else(|| napi::Error::from_reason("matching listener has already been undeclared"))?;

    env.spawn_future(async move { listener.undeclare().await.map_napi_err() })
  }
}
