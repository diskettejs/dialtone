use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;

use crate::{config::*, error::*};

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
  inner: zenoh_ext::SampleMissListener<()>,
}

impl SampleMissListener {
  pub(crate) fn new(inner: zenoh_ext::SampleMissListener<()>) -> Self {
    Self { inner }
  }
}

#[napi]
impl SampleMissListener {
  #[napi(getter)]
  pub fn handler(&self) -> Either<(), ()> {
    todo!()
    // self.receiver.handler()
  }

  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    todo!()
  }
}
