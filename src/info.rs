use napi_derive::napi;
use zenoh::{config as zconfig, session as zsession};

use crate::{error::*, macros::wrapper, protocol::*, qos::*};

#[napi]
pub struct SessionInfo {
  inner: zsession::SessionInfo,
}

impl From<zsession::SessionInfo> for SessionInfo {
  fn from(value: zsession::SessionInfo) -> Self {
    Self { inner: value }
  }
}

#[napi]
impl SessionInfo {
  #[napi]
  pub async fn zid(&self) -> String {
    self.inner.zid().await.to_string()
  }

  #[napi]
  pub async fn routers_zid(&self) -> Vec<String> {
    self
      .inner
      .routers_zid()
      .await
      .map(|zid| zid.to_string())
      .collect()
  }

  #[napi]
  pub async fn peers_zid(&self) -> Vec<String> {
    self
      .inner
      .peers_zid()
      .await
      .map(|zid| zid.to_string())
      .collect()
  }

  #[napi]
  pub async fn transports(&self) -> Vec<Transport> {
    self.inner.transports().await.map(Transport::from).collect()
  }

  #[napi]
  pub async fn links(&self) -> Vec<Link> {
    self.inner.links().await.map(Link::from).collect()
  }
}

wrapper!(zsession::Transport);

#[napi]
impl Transport {
  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.inner.zid().to_string()
  }

  #[napi(getter)]
  pub fn whatami(&self) -> WhatAmI {
    self.inner.whatami().into()
  }

  #[napi(getter)]
  pub fn is_qos(&self) -> bool {
    self.inner.is_qos()
  }

  #[napi(getter)]
  pub fn is_multicast(&self) -> bool {
    self.inner.is_multicast()
  }
}

#[napi(object)]
pub struct LinkPriorities {
  pub min: u8,
  pub max: u8,
}

wrapper!(zsession::Link);

#[napi]
impl Link {
  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.inner.zid().to_string()
  }

  #[napi(getter)]
  pub fn src(&self) -> Locator {
    self.inner.src().clone().into()
  }

  #[napi(getter)]
  pub fn dst(&self) -> Locator {
    self.inner.dst().clone().into()
  }

  #[napi(getter)]
  pub fn group(&self) -> Option<Locator> {
    self.inner.group().cloned().map(Locator::from)
  }

  #[napi(getter)]
  pub fn mtu(&self) -> u16 {
    self.inner.mtu()
  }

  #[napi(getter)]
  pub fn is_streamed(&self) -> bool {
    self.inner.is_streamed()
  }

  #[napi(getter)]
  pub fn interfaces(&self) -> Vec<String> {
    self.inner.interfaces().to_vec()
  }

  #[napi(getter)]
  pub fn auth_identifier(&self) -> Option<String> {
    self.inner.auth_identifier().map(|s| s.to_string())
  }

  #[napi(getter)]
  pub fn priorities(&self) -> Option<LinkPriorities> {
    self
      .inner
      .priorities()
      .map(|(min, max)| LinkPriorities { min, max })
  }

  #[napi(getter)]
  pub fn reliability(&self) -> Option<Reliability> {
    self.inner.reliability().map(Into::into)
  }
}

wrapper!(zconfig::Locator);

#[napi]
impl Locator {
  #[napi(constructor)]
  pub fn new(protocol: String, address: String, metadata: String) -> napi::Result<Self> {
    let inner = zconfig::Locator::new(protocol, address, metadata).map_napi_err()?;
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
    self.inner.to_endpoint().into()
  }

  #[napi]
  pub fn to_endpoint(&self) -> EndPoint {
    self.inner.to_endpoint().into()
  }
}
