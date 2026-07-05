use napi_derive::napi;
use zenoh::{Wait, matching as zmatching};

use crate::{handlers::HandlerImpl, macros::*, utils::*};

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
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    Wait::wait(self.take()?.undeclare()).map_napi_err()
  }
}

recv_handler!(MatchingListener => MatchingStatus);
