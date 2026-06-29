use crate::{config::*, error::*};
use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::handlers as zhandlers;

#[napi]
pub struct Miss {
  inner: zenoh_ext::Miss,
}

impl From<zenoh_ext::Miss> for Miss {
  fn from(inner: zenoh_ext::Miss) -> Self {
    Self { inner }
  }
}

#[napi]
impl Miss {
  #[napi(getter)]
  pub fn source(&self) -> EntityGlobalId {
    self.inner.source().into()
  }

  #[napi(getter)]
  pub fn nb(&self) -> u32 {
    self.inner.nb()
  }
}

#[napi]
pub struct SampleMissListener {
  inner: Option<zenoh_ext::SampleMissListener<()>>,
  receiver: crate::handlers::FifoChannelHandler<zenoh_ext::Miss>,
}

impl SampleMissListener {
  pub(crate) fn new(
    inner: zenoh_ext::SampleMissListener<()>,
    receiver: zhandlers::FifoChannelHandler<zenoh_ext::Miss>,
  ) -> Self {
    Self {
      inner: Some(inner),
      receiver: receiver.into(),
    }
  }
}

#[napi]
impl SampleMissListener {
  #[napi]
  pub async fn recv(&self) -> napi::Result<Miss> {
    self.receiver.recv::<Miss>().await
  }

  #[napi]
  pub fn try_recv(&self) -> napi::Result<Option<Miss>> {
    self.receiver.try_recv::<Miss>()
  }

  #[napi]
  pub fn drain(&self) -> Vec<Miss> {
    self.receiver.drain::<Miss>()
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
  pub fn stream<'env>(&self, env: &'env Env) -> napi::Result<ReadableStream<'env, Miss>> {
    self.receiver.stream::<Miss>(env)
  }

  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    let listener = self.inner.take().ok_or_else(|| {
      napi::Error::from_reason("sample miss listener has already been undeclared")
    })?;

    env.spawn_future(async move { listener.undeclare().await.map_napi_err() })
  }
}
