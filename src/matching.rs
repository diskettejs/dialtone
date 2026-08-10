use derive_more::From;
use napi::bindgen_prelude::AsyncGenerator;
use napi_derive::napi;
use zenoh as z;

use crate::utils::*;

#[napi]
#[derive(From)]
pub struct MatchingStatus(z::matching::MatchingStatus);

#[napi]
impl MatchingStatus {
  #[napi(getter)]
  pub fn matching(&self) -> bool {
    self.0.matching()
  }
}

#[napi]
#[derive(From)]
#[from(forward)]
pub struct MatchingListener(
  Declared<
    z::matching::MatchingListener<z::handlers::FifoChannelHandler<z::matching::MatchingStatus>>,
  >,
);

#[napi]
impl MatchingListener {
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    z::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }

  #[napi]
  pub fn receive(&self) -> napi::Result<MatchingStatusIter> {
    let handler = self.0.get()?.handler().clone();

    Ok(handler.into())
  }
}

/// A stream of the matching status changes reported to a listener.
#[napi(async_iterator)]
#[derive(From)]
pub struct MatchingStatusIter(z::handlers::FifoChannelHandler<z::matching::MatchingStatus>);

#[napi]
impl AsyncGenerator for MatchingStatusIter {
  type Yield = MatchingStatus;
  type Next = ();
  type Return = ();

  fn next(
    &mut self,
    _value: Option<Self::Next>,
  ) -> impl Future<Output = napi::Result<Option<Self::Yield>>> + Send + 'static {
    let receiver = self.0.clone();

    async move {
      match receiver.recv_async().await {
        Ok(status) => Ok(Some(status.into())),
        Err(_) => Ok(None),
      }
    }
  }
}
