use napi_derive::napi;
use zenoh::cancellation as zcancellation;

use crate::error::*;

#[napi]
pub struct CancellationToken {
  inner: zcancellation::CancellationToken,
}

impl From<&CancellationToken> for zcancellation::CancellationToken {
  fn from(value: &CancellationToken) -> Self {
    value.inner.clone()
  }
}

#[napi]
impl CancellationToken {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: zcancellation::CancellationToken::default(),
    }
  }

  #[napi]
  pub async fn cancel(&self) -> napi::Result<()> {
    // Clone the inner token so we don't borrow `&self` across the await.
    let token = self.inner.clone();
    token.cancel().await.map_napi_err()
  }

  /// Returns true if the token was cancelled (i.e. `cancel` was called).
  #[napi(getter)]
  pub fn is_cancelled(&self) -> bool {
    self.inner.is_cancelled()
  }
}
