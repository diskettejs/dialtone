use crate::{config::*, error::*, handlers::HandlerImpl, macros::*};
use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;

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

option_wrapper!(
  zenoh_ext::SampleMissListener<HandlerImpl<zenoh_ext::Miss>>,
  "Undeclared sample miss listener"
);

#[napi]
impl SampleMissListener {
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
