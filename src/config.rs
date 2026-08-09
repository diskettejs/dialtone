use derive_more::{AsRef, From, Into};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh as z;

use crate::utils::*;

#[derive(AsRef, From, Into)]
#[napi]
pub struct EntityGlobalId(z::session::EntityGlobalId);

#[napi]
impl EntityGlobalId {
  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.0.zid().to_string()
  }

  #[napi(getter)]
  pub fn eid(&self) -> u32 {
    self.0.eid()
  }
}

#[derive(AsRef, From, Into)]
#[napi]
pub struct Config(z::config::Config);

#[napi]
impl Config {
  #[napi(factory)]
  pub fn default() -> Self {
    z::config::Config::default().into()
  }

  #[napi]
  pub fn default_config_path_env() -> String {
    z::config::Config::DEFAULT_CONFIG_PATH_ENV.to_string()
  }

  #[napi(factory)]
  pub fn from_env() -> napi::Result<Self> {
    let inner = z::config::Config::from_env().map_napi_err()?;
    Ok(inner.into())
  }

  #[napi(factory)]
  pub fn from_file(path: String) -> napi::Result<Self> {
    let inner = z::config::Config::from_file(path).map_napi_err()?;
    Ok(inner.into())
  }

  #[napi(factory)]
  pub fn from_json5(input: String) -> napi::Result<Self> {
    let inner = z::config::Config::from_json5(&input).map_napi_err()?;
    Ok(inner.into())
  }

  #[napi]
  pub fn get_json(&self, key: String) -> napi::Result<String> {
    self.0.get_json(&key).map_napi_err()
  }

  #[napi]
  pub fn insert_json5(&mut self, key: String, value: String) -> napi::Result<()> {
    self.0.insert_json5(&key, &value).map_napi_err()
  }

  #[napi]
  pub fn remove(&mut self, key: String) -> napi::Result<()> {
    self.0.remove(&key).map_napi_err()
  }
}

#[derive(From)]
#[napi]
pub struct SessionConfig(zenoh_config::GenericConfig);

#[napi]
impl SessionConfig {
  #[napi]
  pub fn get(&self, key: String) -> napi::Result<String> {
    self.0.get(&key).map_napi_err()
  }

  #[napi]
  pub fn insert_json5(&self, key: String, value: String) -> napi::Result<()> {
    self.0.insert_json5(&key, &value).map_napi_err()
  }

  #[napi]
  pub fn to_json(&self) -> String {
    self.0.to_json()
  }

  #[napi]
  pub fn queries_default_timeout_ms(&self) -> BigInt {
    BigInt::from(self.0.queries_default_timeout_ms())
  }

  #[napi]
  pub fn get_plugin_config(&self, plugin_name: String) -> napi::Result<String> {
    self
      .0
      .get_plugin_config(&plugin_name)
      .map(|value| value.to_string())
      .map_napi_err()
  }
}

#[napi(string_enum)]
#[derive(Clone)]
pub enum WhatAmI {
  Router,
  Peer,
  Client,
}

impl From<WhatAmI> for z::config::WhatAmI {
  fn from(value: WhatAmI) -> Self {
    match value {
      WhatAmI::Router => Self::Router,
      WhatAmI::Peer => Self::Peer,
      WhatAmI::Client => Self::Client,
    }
  }
}

impl From<z::config::WhatAmI> for WhatAmI {
  fn from(value: z::config::WhatAmI) -> Self {
    match value {
      z::config::WhatAmI::Router => Self::Router,
      z::config::WhatAmI::Peer => Self::Peer,
      z::config::WhatAmI::Client => Self::Client,
    }
  }
}

#[derive(AsRef, From, Into)]
#[napi]
pub struct WhatAmIMatcher(z::config::WhatAmIMatcher);

#[napi]
impl WhatAmIMatcher {
  #[napi(factory)]
  pub fn empty() -> Self {
    z::config::WhatAmIMatcher::empty().into()
  }

  #[napi]
  pub fn router(&self) -> Self {
    self.0.router().into()
  }

  #[napi]
  pub fn peer(&self) -> Self {
    self.0.peer().into()
  }

  #[napi]
  pub fn client(&self) -> Self {
    self.0.client().into()
  }

  #[napi(getter)]
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  #[napi]
  pub fn matches(&self, w: WhatAmI) -> bool {
    self.0.matches(w.into())
  }

  #[napi]
  pub fn to_str(&self) -> String {
    self.0.to_str().to_string()
  }
}
