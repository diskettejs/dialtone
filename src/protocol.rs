use napi_derive::napi;
use zenoh::config as zconfig;

use crate::{error::*, info::*, macros::*};

enum_mapper!(zconfig::WhatAmI: Router, Peer, Client);

wrapper!(zconfig::WhatAmIMatcher);

#[napi]
impl WhatAmIMatcher {
  #[napi(factory)]
  pub fn empty() -> Self {
    zconfig::WhatAmIMatcher::empty().into()
  }

  #[napi]
  pub fn router(&self) -> Self {
    self.inner.router().into()
  }

  #[napi]
  pub fn peer(&self) -> Self {
    self.inner.peer().into()
  }

  #[napi]
  pub fn client(&self) -> Self {
    self.inner.client().into()
  }

  #[napi(getter)]
  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  #[napi]
  pub fn matches(&self, w: WhatAmI) -> bool {
    self.inner.matches(w.into())
  }

  #[napi]
  pub fn to_str(&self) -> String {
    self.inner.to_str().to_string()
  }
}

wrapper!(zconfig::EndPoint as Metadata);

#[napi]
impl Metadata {
  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.inner.metadata().as_str().to_string()
  }

  #[napi]
  pub fn is_empty(&self) -> bool {
    self.inner.metadata().is_empty()
  }

  #[napi]
  pub fn get(&self, key: String) -> Option<String> {
    self
      .inner
      .metadata()
      .get(&key)
      .map(|value| value.to_string())
  }

  #[napi]
  pub fn values(&self, key: String) -> Vec<String> {
    self
      .inner
      .metadata()
      .values(&key)
      .map(|value| value.to_string())
      .collect()
  }
}

wrapper!(zconfig::EndPoint);

#[napi(object)]
pub struct EndPointParts {
  pub protocol: String,
  pub address: String,
  pub metadata: String,
  pub config: String,
}

#[napi]
impl EndPoint {
  #[napi(constructor)]
  pub fn new(
    protocol: String,
    address: String,
    metadata: String,
    config: String,
  ) -> napi::Result<Self> {
    let inner = zconfig::EndPoint::new(protocol, address, metadata, config).map_napi_err()?;

    Ok(inner.into())
  }

  #[napi(getter)]
  pub fn protocol(&self) -> String {
    self.inner.protocol().as_str().to_string()
  }

  #[napi(getter)]
  pub fn address(&self) -> String {
    self.inner.address().as_str().to_string()
  }

  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.inner.as_str().to_string()
  }

  #[napi]
  pub fn metadata(&self) -> Metadata {
    self.inner.clone().into()
  }

  #[napi]
  pub fn config(&self) -> String {
    self.inner.config().as_str().to_string()
  }

  #[napi]
  pub fn split(&self) -> EndPointParts {
    let (protocol, address, metadata, config) = self.inner.split();
    EndPointParts {
      protocol: protocol.as_str().to_string(),
      address: address.as_str().to_string(),
      metadata: metadata.as_str().to_string(),
      config: config.as_str().to_string(),
    }
  }

  #[napi]
  pub fn to_locator(&self) -> Locator {
    self.inner.to_locator().into()
  }
}
