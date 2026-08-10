use derive_more::From;
use napi::bindgen_prelude::AsyncGenerator;
use napi_derive::napi;

use crate::{config::EntityGlobalId, utils::{Declared, MapNapiErr}};

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
  pub fn receive(&self) -> napi::Result<MissIter> {
    let handler = (**self.0.get()?).clone();

    Ok(handler.into())
  }
}

/// A stream of the missed samples reported to a listener.
#[napi(async_iterator)]
#[derive(From)]
pub struct MissIter(zenoh::handlers::FifoChannelHandler<zenoh_ext::Miss>);

#[napi]
impl AsyncGenerator for MissIter {
  type Yield = Miss;
  type Next = ();
  type Return = ();

  fn next(
    &mut self,
    _value: Option<Self::Next>,
  ) -> impl Future<Output = napi::Result<Option<Self::Yield>>> + Send + 'static {
    let receiver = self.0.clone();

    async move {
      match receiver.recv_async().await {
        Ok(miss) => Ok(Some(miss.into())),
        Err(_) => Ok(None),
      }
    }
  }
}
