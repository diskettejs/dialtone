use derive_more::From;
use napi_derive::napi;

use crate::{config::*, utils::*};

#[napi]
#[derive(From)]
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

#[napi]
#[derive(From)]
#[from(forward)]
pub struct SampleMissListener(
  Declared<zenoh_ext::SampleMissListener<zenoh::handlers::FifoChannelHandler<zenoh_ext::Miss>>>,
);

#[napi]
impl SampleMissListener {
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    zenoh::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }

  #[napi]
  pub async fn recv(&self) -> napi::Result<Miss> {
    let miss = self.0.get()?.recv_async().await.map_napi_err()?;

    Ok(miss.into())
  }

  #[napi]
  pub fn try_recv(&self) -> napi::Result<Option<Miss>> {
    Ok(self.0.get()?.try_recv().map_napi_err()?.map(Into::into))
  }
}
