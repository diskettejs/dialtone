use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::{config as zconfig, handlers::IntoHandler, scouting as zscouting};

use crate::options::ScoutOptions;
use crate::{
  channels::FifoChannel, config::*, error::*, handlers::HandlerImpl, info::*, macros::*,
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
    let ScoutOptions {} = options.unwrap_or_default();

    // NOTE: temp hardcoded because of ongoing channel handlers rework
    let (cb, _receiver) = FifoChannel::new(256).into_handler();

    let builder = zscouting::scout(what, config).with((cb, ()));
    let _scout = builder.await;

    todo!("WIP migration to new generic channel system")
  }

  #[napi(getter)]
  pub fn handler(&self) -> napi::Result<()> {
    todo!()
  }

  #[napi]
  pub fn stop(&mut self) -> napi::Result<()> {
    let scout = self.take()?;

    scout.stop();
    Ok(())
  }
}

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
