use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{
  key_expr as zkey_expr, liveliness as zliveliness, sample as zsample, session as zsession,
};

use crate::{config::*, error::*, key_expr::*, liveliness::*, miss::*, options::*};

#[napi]
pub struct Subscriber {
  inner: zenoh_ext::AdvancedSubscriber<()>,
}

impl Subscriber {
  pub(crate) fn new(inner: zenoh_ext::AdvancedSubscriber<()>) -> Self {
    Self { inner }
  }
}

#[napi]
impl Subscriber {
  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    self.inner.key_expr().clone().into()
  }

  #[napi(getter)]
  pub fn id(&self) -> EntityGlobalId {
    self.inner.id().into()
  }

  #[napi(getter)]
  pub fn handler(&self) -> Either<(), ()> {
    todo!()
  }

  #[napi]
  pub fn sample_miss_listener<'env>(
    &self,
    env: &'env Env,
    options: Option<SampleMissListenerOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, SampleMissListener>> {
    todo!()
  }

  #[napi]
  pub fn detect_publishers<'env>(
    &self,
    env: &'env Env,
    options: Option<LivelinessSubscriberOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, LivelinessSubscriber>> {
    todo!()
  }

  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    todo!()
  }
}
