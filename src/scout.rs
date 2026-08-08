use derive_more::{From, Into};
use napi_derive::napi;
use zenoh as z;

use crate::{config::*, handlers::*, options::*, session::*, utils::*};

#[derive(From)]
#[from(forward)]
#[napi]
pub struct Scout(Declared<Scout, z::scouting::Scout<HandlerImpl<z::scouting::Hello>>>);

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
    let ScoutOptions { channel } = options.unwrap_or_default();
    let handler = into_handler(channel);

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
  pub async fn recv(&self) -> napi::Result<DeferredJs> {
    self.0.get()?.recv().await
  }

  #[napi]
  pub fn try_recv(&self) -> napi::Result<Option<DeferredJs>> {
    self.0.get()?.try_recv()
  }

  #[napi]
  pub fn stream(&self) -> napi::Result<Stream> {
    Ok(self.0.get()?.stream())
  }

  #[napi]
  pub fn handler(&self) -> napi::Result<Handler> {
    Ok(self.0.get()?.share())
  }
}

#[derive(From, Into)]
#[napi]
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
