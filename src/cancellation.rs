use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::error::to_napi_err;

/// A synchronization handle that interrupts the get queries it is attached to
/// (mirrors `zenoh::cancellation::CancellationToken`).
///
/// Create one, pass it to [`Session::get`], [`Querier::get`], or
/// [`Liveliness::get`], then call `cancel` to interrupt every query bound to it.
/// The token is a shared handle: hold on to the same instance to cancel later,
/// and reuse it across multiple gets to cancel them together. Once cancelled, any
/// *new* get it is passed to fails immediately.
///
/// [`Session::get`]: crate::session::Session
/// [`Querier::get`]: crate::querier::Querier
/// [`Liveliness::get`]: crate::liveliness::Liveliness
#[napi]
#[derive(Clone, Default)]
pub struct CancellationToken {
  pub(crate) inner: zenoh::cancellation::CancellationToken,
}

#[napi]
impl CancellationToken {
  /// Create a fresh, un-cancelled token.
  #[napi(constructor)]
  pub fn new() -> Self {
    Self::default()
  }

  /// Interrupt every get query bound to this token. If a reply callback is
  /// mid-execution, resolves only once it finishes; after it resolves, no
  /// further replies are delivered to the bound queries. Idempotent and safe to
  /// call concurrently. Rejects if cancelling some bound operation failed, in
  /// which case not every operation is guaranteed to have been cancelled.
  #[napi]
  pub async fn cancel(&self) -> Result<()> {
    self.inner.cancel().await.map_err(to_napi_err)
  }

  /// Whether `cancel` has been called on this token.
  #[napi(getter)]
  pub fn is_cancelled(&self) -> bool {
    self.inner.is_cancelled()
  }
}
