use derive_more::From;
use napi::bindgen_prelude::AsyncGenerator;
use napi_derive::napi;
use zenoh as z;

use crate::utils::{Declared, MapNapiErr};

/// Whether there exist entities matching a publisher or a querier.
#[napi]
#[derive(From)]
pub struct MatchingStatus(z::matching::MatchingStatus);

#[napi]
impl MatchingStatus {
  /// `true` if there exist entities matching the declaring entity, i.e. subscribers
  /// matching a publisher's key expression, or queryables matching a querier's key
  /// expression and target.
  #[napi(getter)]
  pub fn matching(&self) -> bool {
    self.0.matching()
  }
}

/// A listener notified each time the {@link `MatchingStatus`} of the entity that declared
/// it changes.
///
/// {@link MatchingListener.undeclare} consumes the listener; every member throws once it
/// has been undeclared.
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
  /// Undeclares this listener, so that no further matching status is reported to it.
  ///
  /// @throws If this listener has already been undeclared.
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    z::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }

  /// Iterates over the matching status changes reported to this listener.
  ///
  /// @returns A {@link `MatchingStatusIter`} that yields each buffered
  /// {@link `MatchingStatus`} and completes once the listener stops receiving.
  /// @throws If this listener has already been undeclared.
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
