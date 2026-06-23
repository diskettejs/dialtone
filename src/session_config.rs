use napi::bindgen_prelude::BigInt;
use napi_derive::napi;

/// The live configuration sub-API for a session, reached via `Session.config()`.
///
/// Distinct from the [`Config`](crate::config::Config) used to *open* a session
/// (an owned, pre-open snapshot): this is a live handle onto the running
/// session's configuration. `get` / `getPluginConfig` read the current values
/// and `insertJson5` reconfigures the session in place. Mirrors zenoh's
/// `GenericConfig`.
///
/// Like the other session sub-APIs, this holds a clone of the session and reads
/// the live config (`session.config()`) per call; the surface methods thread
/// through zenoh's `IConfig` without requiring its config internals to be named.
#[napi]
pub struct SessionConfig {
  session: zenoh::Session,
}

impl SessionConfig {
  pub(crate) fn from_session(session: zenoh::Session) -> Self {
    SessionConfig { session }
  }
}

#[napi]
impl SessionConfig {
  /// Reads the configuration value at `key`, returned as a JSON string.
  #[napi]
  pub fn get(&self, key: String) -> napi::Result<String> {
    self
      .session
      .config()
      .get(&key)
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }

  /// Inserts the JSON5 `value` at `key`, reconfiguring the running session.
  #[napi]
  pub fn insert_json5(&self, key: String, value: String) -> napi::Result<()> {
    self
      .session
      .config()
      .insert_json5(&key, &value)
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }

  /// The full current configuration, as a JSON string.
  #[napi]
  pub fn to_json(&self) -> String {
    self.session.config().to_json()
  }

  /// The default timeout applied to queries, in milliseconds.
  #[napi]
  pub fn queries_default_timeout_ms(&self) -> BigInt {
    BigInt::from(self.session.config().queries_default_timeout_ms())
  }

  /// Reads the configuration of plugin `pluginName`, returned as a JSON string.
  #[napi]
  pub fn get_plugin_config(&self, plugin_name: String) -> napi::Result<String> {
    self
      .session
      .config()
      .get_plugin_config(&plugin_name)
      .map(|value| value.to_string())
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }
}
