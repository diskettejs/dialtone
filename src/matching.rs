use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::matching as zmatching;

use crate::{error::*, handlers::HandlerImpl, macros::*};

wrapper!(zmatching::MatchingStatus);

#[napi]
impl MatchingStatus {
  #[napi(getter)]
  pub fn matching(&self) -> bool {
    self.inner.matching()
  }
}

option_wrapper!(
  zmatching::MatchingListener<HandlerImpl<zmatching::MatchingStatus>>,
  "Undeclared matching listener"
);

#[napi]
impl MatchingListener {
  #[napi(getter)]
  pub fn handler(&self) -> napi::Result<()> {
    todo!()
  }

  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    let listener = self.take()?;

    env.spawn_future(async move { listener.undeclare().await.map_napi_err() })
  }
}
