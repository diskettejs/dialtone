use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::{config as zconfig, handlers::IntoHandler, scouting as zscouting};

use crate::options::ScoutOptions;
use crate::{
  config::*,
  error::*,
  handlers::{HandlerImpl, into_handler},
  info::*,
  macros::*,
  protocol::*,
};

option_wrapper!(
  zscouting::Scout<HandlerImpl<zscouting::Hello>>,
  "Stopped scout"
);

#[napi]
#[allow(clippy::self_named_constructors)]
impl Scout {
  #[napi]
  pub async fn scout(
    what: &WhatAmIMatcher,
    config: &Config,
    options: Option<ScoutOptions>,
  ) -> napi::Result<Scout> {
    let what: zconfig::WhatAmIMatcher = what.into();
    let config: zconfig::Config = config.into();
    let ScoutOptions { channel } = options.unwrap_or_default();
    let handler = into_handler(channel);

    let scout = zscouting::scout(what, config)
      .with(handler)
      .await
      .map_napi_err()?;

    Ok(scout.into())
  }

  #[napi]
  pub fn stop(&mut self) -> napi::Result<()> {
    let scout = self.take()?;

    scout.stop();
    Ok(())
  }
}

recv_handler!(Scout => Hello);

wrapper!(zscouting::Hello);

#[napi]
impl Hello {
  #[napi]
  pub fn locators(&self) -> Vec<Locator> {
    self
      .inner
      .locators()
      .iter()
      .map(|l| l.clone().into())
      .collect()
  }

  #[napi(getter)]
  pub fn whatami(&self) -> WhatAmI {
    self.inner.whatami().into()
  }

  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.inner.zid().to_string()
  }
}
