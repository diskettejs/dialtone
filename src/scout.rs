use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::handlers::fifo as zfifo;
use zenoh::{config as zconfig, handlers::IntoHandler, sample as zsample, scouting as zscouting};

use crate::{channels::FifoChannel, config::*, error::*, info::*, options::*, protocol::*};

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
impl Scout {
  #[napi]
  pub fn scout<'env>(
    env: &'env Env,
    what: &WhatAmIMatcher,
    config: &Config,
  ) -> napi::Result<PromiseRaw<'env, Scout>> {
    let what: zconfig::WhatAmIMatcher = what.into();
    let config: zconfig::Config = config.into();
    let (cb, receiver) = FifoChannel::default().into_handler();

    env.spawn_future(async move {
      let scout = zscouting::scout(what, config)
        .with((cb, ()))
        .await
        .map_napi_err()?;

      Ok(Scout::new(scout, receiver))
    })
  }

  #[napi]
  pub async fn recv(&self) -> napi::Result<Hello> {
    self.receiver.recv::<Hello>().await
  }

  #[napi]
  pub fn try_recv(&self) -> napi::Result<Option<Hello>> {
    self.receiver.try_recv::<Hello>()
  }

  #[napi]
  pub fn drain(&self) -> Vec<Hello> {
    self.receiver.drain::<Hello>()
  }

  #[napi]
  pub fn is_disconnected(&self) -> bool {
    self.receiver.is_disconnected()
  }

  #[napi]
  pub fn is_empty(&self) -> bool {
    self.receiver.is_empty()
  }

  #[napi]
  pub fn is_full(&self) -> bool {
    self.receiver.is_full()
  }

  #[napi]
  pub fn len(&self) -> u32 {
    self.receiver.len()
  }

  #[napi]
  pub fn capacity(&self) -> Option<u32> {
    self.receiver.capacity()
  }

  #[napi]
  pub fn sender_count(&self) -> u32 {
    self.receiver.sender_count()
  }

  #[napi]
  pub fn receiver_count(&self) -> u32 {
    self.receiver.receiver_count()
  }

  #[napi]
  pub fn stream<'env>(&self, env: &'env Env) -> napi::Result<ReadableStream<'env, Hello>> {
    self.receiver.stream::<Hello>(env)
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
