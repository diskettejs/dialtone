use derive_more::From;
use napi_derive::napi;

use crate::{config::*, handlers::*, utils::*};

#[derive(From)]
#[napi]
pub struct Miss(zenoh_ext::Miss);

#[napi]
impl Miss {
  #[napi(getter)]
  pub fn source(&self) -> EntityGlobalId {
    self.0.source().into()
  }

  #[napi(getter)]
  pub fn nb(&self) -> u32 {
    self.0.nb()
  }
}

#[derive(From)]
#[from(forward)]
#[napi]
pub struct SampleMissListener(
  Declared<SampleMissListener, zenoh_ext::SampleMissListener<HandlerImpl<zenoh_ext::Miss>>>,
);

#[napi]
impl SampleMissListener {
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    zenoh::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }

  #[napi]
  pub async fn recv(&self) -> napi::Result<DeferredJs> {
    self.0.get()?.recv().await
  }

  #[napi]
  pub fn try_recv(&self) -> napi::Result<Option<DeferredJs>> {
    self.0.get()?.try_recv()
  }

  #[napi]
  pub fn stream(&self) -> napi::Result<Stream> {
    Ok(self.0.get()?.stream())
  }

  #[napi]
  pub fn handler(&self) -> napi::Result<Handler> {
    Ok(self.0.get()?.share())
  }
}
