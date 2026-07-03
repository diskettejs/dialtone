use crate::{config::*, error::*, handlers::HandlerImpl, macros::*};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::Wait;

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
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    Wait::wait(self.take()?.undeclare()).map_napi_err()
  }
}

recv_handler!(SampleMissListener => Miss);
