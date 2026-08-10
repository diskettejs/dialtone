use derive_more::From;
use napi::bindgen_prelude::AsyncGenerator;
use napi_derive::napi;
use zenoh as z;

use crate::{config::{WhatAmIMatcher, Config, WhatAmI}, options::ScoutOptions, session::Locator, utils::{Declared, fifo, MapNapiErr}};

#[napi]
#[derive(From)]
#[from(forward)]
pub struct Scout(Declared<z::scouting::Scout<z::handlers::FifoChannelHandler<z::scouting::Hello>>>);

#[napi]
#[allow(clippy::self_named_constructors)]
impl Scout {
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

  #[napi]
  pub fn stop(&mut self) -> napi::Result<()> {
    let scout = self.0.take()?;

    scout.stop();
    Ok(())
  }

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

#[napi]
#[derive(From)]
pub struct Hello(z::scouting::Hello);

#[napi]
impl Hello {
  #[napi]
  pub fn locators(&self) -> Vec<Locator> {
    self.0.locators().iter().map(|l| l.clone().into()).collect()
  }

  #[napi(getter)]
  pub fn whatami(&self) -> WhatAmI {
    self.0.whatami().into()
  }

  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.0.zid().to_string()
  }
}
