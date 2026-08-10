use derive_more::{From, Into};
use napi_derive::napi;
use zenoh as z;

use crate::{config::*, options::*, session::*, utils::*};

#[derive(From)]
#[from(forward)]
#[napi]
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
  pub async fn recv(&self) -> napi::Result<Hello> {
    let hello = self.0.get()?.recv_async().await.map_napi_err()?;

    Ok(hello.into())
  }

  #[napi]
  pub fn try_recv(&self) -> napi::Result<Option<Hello>> {
    Ok(self.0.get()?.try_recv().map_napi_err()?.map(Into::into))
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
