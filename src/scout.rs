use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::handlers::fifo as zfifo;
use zenoh::{config as zconfig, handlers::IntoHandler, scouting as zscouting};

use crate::options::ScoutOptions;
use crate::{
  channels::FifoChannel,
  config::*,
  error::*,
  info::*,
  macros::{channel_forward, wrapper},
  protocol::*,
};

#[napi]
pub struct Scout {
  inner: Option<zscouting::Scout<()>>,
  receiver: crate::handlers::FifoChannelHandler<zscouting::Hello>,
}

impl Scout {
  pub fn new(
    scout: zscouting::Scout<()>,
    receiver: zfifo::FifoChannelHandler<zscouting::Hello>,
  ) -> Self {
    Scout {
      inner: Some(scout),
      receiver: receiver.into(),
    }
  }
}

#[napi]
#[allow(clippy::self_named_constructors)]
impl Scout {
  #[napi]
  pub fn scout<'env>(
    env: &'env Env,
    what: &WhatAmIMatcher,
    config: &Config,
    options: Option<ScoutOptions>,
  ) -> napi::Result<PromiseRaw<'env, Scout>> {
    let what: zconfig::WhatAmIMatcher = what.into();
    let config: zconfig::Config = config.into();
    let ScoutOptions { capacity } = options.unwrap_or_default();
    let (cb, receiver) = FifoChannel::with_capacity(capacity).into_handler();

    env.spawn_future(async move {
      let scout = zscouting::scout(what, config)
        .with((cb, ()))
        .await
        .map_napi_err()?;

      Ok(Scout::new(scout, receiver))
    })
  }

  #[napi]
  pub fn stop(&mut self) -> napi::Result<()> {
    let scout = self
      .inner
      .take()
      .ok_or_else(|| napi::Error::from_reason("scout has already been stopped"))?;

    scout.stop();
    Ok(())
  }
}

channel_forward!(Scout, receiver, Hello);

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
