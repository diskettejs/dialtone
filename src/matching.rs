use derive_more::From;
use napi_derive::napi;
use zenoh as z;

use crate::{handlers::*, utils::*};

#[derive(From)]
#[napi]
pub struct MatchingStatus(z::matching::MatchingStatus);

#[napi]
impl MatchingStatus {
  #[napi(getter)]
  pub fn matching(&self) -> bool {
    self.0.matching()
  }
}

#[derive(From)]
#[from(forward)]
#[napi]
pub struct MatchingListener(
  Declared<z::matching::MatchingListener<HandlerImpl<z::matching::MatchingStatus>>>,
);

#[napi]
impl MatchingListener {
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    z::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }

  #[napi]
  pub async fn recv(&self) -> napi::Result<DeferredJs> {
    self.0.get()?.recv().await
  }

  #[napi]
  pub fn try_recv(&self) -> napi::Result<Option<DeferredJs>> {
    self.0.get()?.try_recv()
  }
}
