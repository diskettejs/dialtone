use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::matching as zmatching;

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
  inner: zmatching::MatchingListener<()>,
}

impl MatchingListener {
  pub(crate) fn new(inner: zmatching::MatchingListener<()>) -> Self {
    Self { inner }
  }
}

#[napi]
impl MatchingListener {
  #[napi(getter)]
  pub fn handler(&self) -> Either<(), ()> {
    todo!()
  }

  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    todo!()
  }
}
