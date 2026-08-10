use derive_more::{AsRef, From};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh as z;

use crate::utils::MapNapiErr;

/// The identifier globally identifying an entity in a Zenoh system.
#[napi]
#[derive(From)]
pub struct EntityGlobalId(z::session::EntityGlobalId);

#[napi]
impl EntityGlobalId {
  /// The Zenoh identifier of the session this entity belongs to.
  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.0.zid().to_string()
  }

  /// The identifier of this entity within its session.
  #[napi(getter)]
  pub fn eid(&self) -> u32 {
    self.0.eid()
  }
}

/// The configuration a session is opened with.
///
/// The configuration tree has no stable shape, so its fields are not exposed individually.
/// A configuration is loaded from a file or a JSON5 string with {@link Config.fromFile} or
/// {@link Config.fromJson5}, and edited with {@link Config.insertJson5} and
/// {@link Config.remove}.
#[napi]
#[derive(AsRef, From)]
pub struct Config(z::config::Config);

#[napi]
impl Config {
  /// Returns the default configuration.
  #[napi(factory)]
  pub fn default() -> Self {
    z::config::Config::default().into()
  }

  /// Returns the name of the environment variable {@link Config.fromEnv} reads the
  /// configuration file path from.
  #[napi]
  pub fn default_config_path_env() -> String {
    z::config::Config::DEFAULT_CONFIG_PATH_ENV.to_string()
  }

  /// Loads the configuration from the file whose path is held by the environment variable
  /// named by {@link Config.defaultConfigPathEnv}.
  ///
  /// @throws If the variable is unset, or if the file cannot be read or is not a valid
  /// configuration.
  #[napi(factory)]
  pub fn from_env() -> napi::Result<Self> {
    let inner = z::config::Config::from_env().map_napi_err()?;
    Ok(inner.into())
  }

  /// Loads the configuration from the file at `path`.
  ///
  /// @throws If the file cannot be read or is not a valid configuration.
  #[napi(factory)]
  pub fn from_file(path: String) -> napi::Result<Self> {
    let inner = z::config::Config::from_file(path).map_napi_err()?;
    Ok(inner.into())
  }

  /// Loads the configuration from the JSON5 string `input`.
  ///
  /// @throws If the string is not a valid configuration.
  #[napi(factory)]
  pub fn from_json5(input: String) -> napi::Result<Self> {
    let inner = z::config::Config::from_json5(&input).map_napi_err()?;
    Ok(inner.into())
  }

  /// Returns the configuration held at `key`, as a JSON string.
  ///
  /// @throws If no configuration is held at `key`.
  #[napi]
  pub fn get_json(&self, key: String) -> napi::Result<String> {
    self.0.get_json(&key).map_napi_err()
  }

  /// Inserts the JSON5 `value` at `key`.
  ///
  /// A key of the form `<path>/<idKey>=<idValue>` addresses one item of the list held at
  /// `<path>`, the one whose `<idKey>` field equals `<idValue>`. The item is appended when
  /// the list holds no such item yet.
  ///
  /// @throws If `key` does not address an insertable entry, or if `value` is not valid at
  /// that key.
  #[napi]
  pub fn insert_json5(&mut self, key: String, value: String) -> napi::Result<()> {
    self.0.insert_json5(&key, &value).map_napi_err()
  }

  /// Removes the configuration held at `key`.
  ///
  /// A key of the form `<path>/<idKey>=<idValue>` addresses one item of the list held at
  /// `<path>`, the one whose `<idKey>` field equals `<idValue>`.
  ///
  /// @throws If `key` does not address a removable entry.
  #[napi]
  pub fn remove(&mut self, key: String) -> napi::Result<()> {
    self.0.remove(&key).map_napi_err()
  }
}

/// The configuration an open {@link `Session`} is currently running with.
///
/// It reads the whole configuration of the session, and applies changes to the plugin part
/// of it.
#[napi]
#[derive(From)]
pub struct SessionConfig(zenoh_config::GenericConfig);

#[napi]
impl SessionConfig {
  /// Returns the configuration held at `key`, as a JSON string.
  ///
  /// @throws If no configuration is held at `key`.
  #[napi]
  pub fn get(&self, key: String) -> napi::Result<String> {
    self.0.get(&key).map_napi_err()
  }

  /// Inserts the JSON5 `value` at `key`, applying it to the running session.
  ///
  /// Only keys under `plugins/` can be updated on a running session.
  ///
  /// @throws If `key` is outside `plugins/`, or if `value` is not valid at that key.
  #[napi]
  pub fn insert_json5(&self, key: String, value: String) -> napi::Result<()> {
    self.0.insert_json5(&key, &value).map_napi_err()
  }

  /// Returns the whole configuration as a JSON string.
  #[napi]
  pub fn to_json(&self) -> String {
    self.0.to_json()
  }

  /// Returns the default timeout, in milliseconds, applied to the queries issued by this
  /// session.
  #[napi]
  pub fn queries_default_timeout_ms(&self) -> BigInt {
    BigInt::from(self.0.queries_default_timeout_ms())
  }

  /// Returns the configuration of the plugin named `pluginName`, as a JSON string.
  ///
  /// @throws If the plugin holds no configuration.
  #[napi]
  pub fn get_plugin_config(&self, plugin_name: String) -> napi::Result<String> {
    self
      .0
      .get_plugin_config(&plugin_name)
      .map(|value| value.to_string())
      .map_napi_err()
  }
}

/// The kind of a node in the Zenoh network.
///
/// A peer searches for the other nodes and establishes direct connections with them, a
/// client stays connected to a single node that gateways it to the rest of the network,
/// and a router maintains a statically configured network topology.
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

/// A set of {@link `WhatAmI`} values, used to select the kinds of node a scout looks for.
///
/// Start from {@link WhatAmIMatcher.empty} and add the kinds to match with
/// {@link WhatAmIMatcher.router}, {@link WhatAmIMatcher.peer} and
/// {@link WhatAmIMatcher.client}.
#[napi]
#[derive(AsRef, From)]
pub struct WhatAmIMatcher(z::config::WhatAmIMatcher);

#[napi]
impl WhatAmIMatcher {
  /// Returns a matcher that matches no node kind.
  #[napi(factory)]
  pub fn empty() -> Self {
    z::config::WhatAmIMatcher::empty().into()
  }

  /// Returns a copy of this matcher that also matches routers.
  #[napi]
  pub fn router(&self) -> Self {
    self.0.router().into()
  }

  /// Returns a copy of this matcher that also matches peers.
  #[napi]
  pub fn peer(&self) -> Self {
    self.0.peer().into()
  }

  /// Returns a copy of this matcher that also matches clients.
  #[napi]
  pub fn client(&self) -> Self {
    self.0.client().into()
  }

  /// Whether this matcher matches no node kind.
  #[napi(getter)]
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  /// Returns whether this matcher matches the given node kind.
  #[napi]
  pub fn matches(&self, w: WhatAmI) -> bool {
    self.0.matches(w.into())
  }

  /// Returns the matched node kinds as their names joined by `|`, e.g. `router|peer`.
  #[napi]
  pub fn to_str(&self) -> String {
    self.0.to_str().to_string()
  }
}
