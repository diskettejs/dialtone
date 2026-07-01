use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::{config as zconfig, session as zsession};

use crate::{error::*, macros::wrapper};

wrapper!(zsession::EntityGlobalId);

#[napi]
impl EntityGlobalId {
  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.inner.zid().to_string()
  }

  #[napi(getter)]
  pub fn eid(&self) -> u32 {
    self.inner.eid()
  }
}

wrapper!(zconfig::Config);

#[napi]
impl Config {
  #[napi(factory)]
  pub fn default() -> Self {
    Self {
      inner: zconfig::Config::default(),
    }
  }

  #[napi]
  pub fn default_config_path_env() -> String {
    zconfig::Config::DEFAULT_CONFIG_PATH_ENV.to_string()
  }

  #[napi(factory)]
  pub fn from_env() -> napi::Result<Self> {
    let inner = zconfig::Config::from_env().map_napi_err()?;
    Ok(Config { inner })
  }

  #[napi(factory)]
  pub fn from_file(path: String) -> napi::Result<Self> {
    let inner = zconfig::Config::from_file(path).map_napi_err()?;
    Ok(Config { inner })
  }

  #[napi(factory)]
  pub fn from_json5(input: String) -> napi::Result<Self> {
    let inner = zconfig::Config::from_json5(&input).map_napi_err()?;
    Ok(Config { inner })
  }

  #[napi]
  pub fn get_json(&self, key: String) -> napi::Result<String> {
    self.inner.get_json(&key).map_napi_err()
  }

  #[napi]
  pub fn insert_json5(&mut self, key: String, value: String) -> napi::Result<()> {
    self.inner.insert_json5(&key, &value).map_napi_err()
  }

  #[napi]
  pub fn remove(&mut self, key: String) -> napi::Result<()> {
    self.inner.remove(&key).map_napi_err()
  }
}

#[napi]
pub struct SessionConfig {
  inner: zenoh_config::GenericConfig,
}

impl From<zenoh_config::GenericConfig> for SessionConfig {
  fn from(value: zenoh_config::GenericConfig) -> Self {
    Self { inner: value }
  }
}

#[napi]
impl SessionConfig {
  #[napi]
  pub fn get(&self, key: String) -> napi::Result<String> {
    self.inner.get(&key).map_napi_err()
  }

  #[napi]
  pub fn insert_json5(&self, key: String, value: String) -> napi::Result<()> {
    self.inner.insert_json5(&key, &value).map_napi_err()
  }

  #[napi]
  pub fn to_json(&self) -> String {
    self.inner.to_json()
  }

  #[napi]
  pub fn queries_default_timeout_ms(&self) -> BigInt {
    BigInt::from(self.inner.queries_default_timeout_ms())
  }

  #[napi]
  pub fn get_plugin_config(&self, plugin_name: String) -> napi::Result<String> {
    self
      .inner
      .get_plugin_config(&plugin_name)
      .map(|value| value.to_string())
      .map_napi_err()
  }
}
