use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{config as zconfig, scouting as zscouting};

use crate::{config::*, error::*, info::*, options::*, protocol::*};

#[napi]
pub struct Scout {
  inner: zscouting::Scout<()>,
}

#[napi]
impl Scout {
  #[napi]
  pub fn scout<'env>(
    env: &'env Env,
    what: &WhatAmIMatcher,
    config: &Config,
    options: Option<ScoutOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, Scout>> {
    todo!()
    // let what: zconfig::WhatAmIMatcher = what.into();
    // let config: zconfig::Config = config.into();
    // env.spawn_future(async move {
    //   let scout = zscouting::scout(what, config)
    //     .with(callback)
    //     .await
    //     .map_napi_err()?;
    //   Ok(Scout {
    //     inner: Declared::new(scout),
    //     receiver,
    //   })
    // })
  }

  #[napi(getter)]
  pub fn handler(&self) -> Either<(), ()> {
    todo!()
  }

  #[napi]
  pub fn stop(&mut self) -> napi::Result<()> {
    todo!()
  }
}

#[napi]
pub struct Hello {
  inner: zscouting::Hello,
}

impl From<zscouting::Hello> for Hello {
  fn from(inner: zscouting::Hello) -> Self {
    Self { inner }
  }
}

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
