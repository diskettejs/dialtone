use derive_more::From;
use napi_derive::napi;

use crate::utils::MapNapiErr;

#[napi]
#[derive(Clone, From)]
pub struct CancellationToken(zenoh::cancellation::CancellationToken);

#[napi]
impl CancellationToken {
  #[napi(constructor)]
  pub fn new() -> Self {
    zenoh::cancellation::CancellationToken::default().into()
  }

  #[napi]
  pub async fn cancel(&self) -> napi::Result<()> {
    // Clone the inner token so we don't borrow `&self` across the await.
    let token = self.0.clone();
    token.cancel().await.map_napi_err()
  }

  /// Returns true if the token was cancelled (i.e. `cancel` was called).
  #[napi(getter)]
  pub fn is_cancelled(&self) -> bool {
    self.0.is_cancelled()
  }
}
