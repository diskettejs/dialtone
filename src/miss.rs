use crate::{
  config::*,
  error::*,
  macros::{channel_forward, wrapper},
};
use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::handlers as zhandlers;

wrapper!(zenoh_ext::Miss);

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
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    let listener = self.inner.take().ok_or_else(|| {
      napi::Error::from_reason("sample miss listener has already been undeclared")
    })?;

    env.spawn_future(async move { listener.undeclare().await.map_napi_err() })
  }
}

channel_forward!(SampleMissListener, receiver, Miss);
