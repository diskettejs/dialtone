use derive_more::From;
use napi::bindgen_prelude::AsyncGenerator;
use napi_derive::napi;
use zenoh as z;

use crate::{
  config::{Config, WhatAmI, WhatAmIMatcher},
  options::ScoutOptions,
  session::Locator,
  utils::{Declared, MapNapiErr, fifo},
};

/// A running scout, discovering the Zenoh processes reachable on the network.
///
/// Scouting periodically sends scout messages and collects the {@link `Hello`} replies of
/// the processes that answer them.
///
/// {@link Scout.stop} consumes the scout; every member throws once it has been stopped.
#[napi]
#[derive(From)]
#[from(forward)]
pub struct Scout(Declared<z::scouting::Scout<z::handlers::FifoChannelHandler<z::scouting::Hello>>>);

#[napi]
#[allow(clippy::self_named_constructors)]
impl Scout {
  /// Starts scouting for the kinds of Zenoh process selected by `what`, using `config`
  /// for the scouting parameters such as the multicast address and interfaces.
  #[napi]
  pub async fn scout(
    what: &WhatAmIMatcher,
    config: &Config,
    options: Option<ScoutOptions>,
  ) -> napi::Result<Scout> {
    let what: z::config::WhatAmIMatcher = *what.as_ref();
    let config = config.as_ref().clone();
    let ScoutOptions { channel_capacity } = options.unwrap_or_default();
    let handler = fifo(channel_capacity);

    let scout = z::scouting::scout(what, config)
      .with(handler)
      .await
      .map_napi_err()?;

    Ok(scout.into())
  }

  /// Stops scouting.
  ///
  /// @throws If this scout has already been stopped.
  #[napi]
  pub fn stop(&mut self) -> napi::Result<()> {
    let scout = self.0.take()?;

    scout.stop();
    Ok(())
  }

  /// Iterates over the hellos received while scouting.
  ///
  /// @returns A {@link `HelloIter`} that yields each buffered hello and completes once
  /// scouting stops.
  /// @throws If this scout has already been stopped.
  #[napi]
  pub fn receive(&self) -> napi::Result<HelloIter> {
    let handler = (**self.0.get()?).clone();

    Ok(handler.into())
  }
}

/// A stream of the hellos received while scouting.
#[napi(async_iterator)]
#[derive(From)]
pub struct HelloIter(z::handlers::FifoChannelHandler<z::scouting::Hello>);

#[napi]
impl AsyncGenerator for HelloIter {
  type Yield = Hello;
  type Next = ();
  type Return = ();

  fn next(
    &mut self,
    _value: Option<Self::Next>,
  ) -> impl Future<Output = napi::Result<Option<Self::Yield>>> + Send + 'static {
    let receiver = self.0.clone();

    async move {
      match receiver.recv_async().await {
        Ok(hello) => Ok(Some(hello.into())),
        Err(_) => Ok(None),
      }
    }
  }
}

/// The reply a Zenoh process sends to a scout message, announcing itself.
#[napi]
#[derive(From)]
pub struct Hello(z::scouting::Hello);

#[napi]
impl Hello {
  /// The locators this process can be reached at.
  #[napi]
  pub fn locators(&self) -> Vec<Locator> {
    self.0.locators().iter().map(|l| l.clone().into()).collect()
  }

  /// The kind of Zenoh process that sent this hello.
  #[napi(getter)]
  pub fn whatami(&self) -> WhatAmI {
    self.0.whatami().into()
  }

  /// The Zenoh identifier of the process that sent this hello.
  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.0.zid().to_string()
  }
}
