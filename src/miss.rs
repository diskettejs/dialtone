use derive_more::From;
use napi::bindgen_prelude::AsyncGenerator;
use napi_derive::napi;

use crate::{
  config::EntityGlobalId,
  utils::{Declared, MapNapiErr},
};

/// A report of samples a subscriber did not receive.
#[napi]
#[derive(From)]
pub struct Miss(zenoh_ext::Miss);

#[napi]
impl Miss {
  /// The global identifier of the publisher whose samples were missed.
  #[napi(getter)]
  pub fn source(&self) -> EntityGlobalId {
    self.0.source().into()
  }

  /// The number of samples missed from that publisher.
  #[napi(getter)]
  pub fn nb(&self) -> u32 {
    self.0.nb()
  }
}

/// A listener reporting the samples a subscriber missed.
///
/// Missed samples can only be detected from publishers that enable
/// {@link PublisherOptions.sampleMissDetection}.
///
/// {@link SampleMissListener.undeclare} consumes the listener; every member throws once it
/// has been undeclared.
#[napi]
#[derive(From)]
#[from(forward)]
pub struct SampleMissListener(
  Declared<zenoh_ext::SampleMissListener<zenoh::handlers::FifoChannelHandler<zenoh_ext::Miss>>>,
);

#[napi]
impl SampleMissListener {
  /// Undeclares this listener, so that no further miss is reported to it.
  ///
  /// @throws If this listener has already been undeclared.
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    zenoh::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }

  /// Iterates over the misses reported to this listener.
  ///
  /// @returns A {@link MissIter} that yields each buffered {@link Miss} and completes
  /// once the listener stops receiving.
  /// @throws If this listener has already been undeclared.
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
