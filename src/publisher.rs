use std::sync::Arc;

use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{
  bytes as zbytes,
  handlers::IntoHandler,
  key_expr as zkey_expr, qos as zqos, session as zsession, time as ztime,
};

use crate::{
  bytes::*, channels::FifoChannel, config::*, encoding::*, error::*, key_expr::*, matching::*,
  options::*, qos::*,
};

#[napi]
pub struct Publisher {
  id: zsession::EntityGlobalId,
  key_expr: zkey_expr::KeyExpr<'static>,
  encoding: zbytes::Encoding,
  congestion_control: zqos::CongestionControl,
  priority: zqos::Priority,
  inner: Option<Arc<zenoh_ext::AdvancedPublisher<'static>>>,
}

impl From<zenoh_ext::AdvancedPublisher<'static>> for Publisher {
  fn from(inner: zenoh_ext::AdvancedPublisher<'static>) -> Self {
    Self {
      id: inner.id(),
      key_expr: inner.key_expr().clone(),
      encoding: inner.encoding().clone(),
      congestion_control: inner.congestion_control(),
      priority: inner.priority(),
      inner: Some(Arc::new(inner)),
    }
  }
}

impl Publisher {
  fn get(&self) -> napi::Result<&zenoh_ext::AdvancedPublisher<'static>> {
    self
      .inner
      .as_deref()
      .ok_or_else(|| napi::Error::from_reason("publisher has already been undeclared"))
  }

  fn arc(&self) -> napi::Result<Arc<zenoh_ext::AdvancedPublisher<'static>>> {
    self
      .inner
      .as_ref()
      .map(Arc::clone)
      .ok_or_else(|| napi::Error::from_reason("publisher has already been undeclared"))
  }
}

#[napi]
impl Publisher {
  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    self.key_expr.clone().into()
  }

  #[napi(getter)]
  pub fn id(&self) -> EntityGlobalId {
    self.id.into()
  }

  #[napi(getter)]
  pub fn encoding(&self) -> Encoding {
    self.encoding.clone().into()
  }

  #[napi(getter)]
  pub fn congestion_control(&self) -> CongestionControl {
    self.congestion_control.into()
  }

  #[napi(getter)]
  pub fn priority(&self) -> Priority {
    self.priority.into()
  }

  #[napi]
  pub fn put<'env>(
    &self,
    env: &'env Env,
    payload: PayloadArg,
    options: Option<PublisherPutOptions>,
  ) -> napi::Result<PromiseRaw<'env, ()>> {
    let payload = payload.into_zbytes();
    let PublisherPutOptions {
      encoding,
      timestamp,
      attachment,
    } = options.unwrap_or_default();
    let encoding = encoding.map(zbytes::Encoding::from);
    let timestamp = timestamp.map(|ts| ztime::Timestamp::from(ts.as_ref()));
    let attachment = attachment.map(IntoZBytes::into_zbytes);
    let publisher = self.arc()?;

    env.spawn_future(async move {
      let mut builder = publisher.put(payload);

      if let Some(encoding) = encoding {
        builder = builder.encoding(encoding);
      }

      if let Some(timestamp) = timestamp {
        builder = builder.timestamp(timestamp);
      }

      if let Some(attachment) = attachment {
        builder = builder.attachment(attachment);
      }

      builder.await.map_napi_err()
    })
  }

  #[napi]
  pub fn delete<'env>(
    &self,
    env: &'env Env,
    options: Option<PublisherDeleteOptions>,
  ) -> napi::Result<PromiseRaw<'env, ()>> {
    let PublisherDeleteOptions {
      timestamp,
      attachment,
    } = options.unwrap_or_default();
    let timestamp = timestamp.map(|ts| ztime::Timestamp::from(ts.as_ref()));
    let attachment = attachment.map(IntoZBytes::into_zbytes);
    let publisher = self.arc()?;

    env.spawn_future(async move {
      let mut builder = publisher.delete();

      if let Some(timestamp) = timestamp {
        builder = builder.timestamp(timestamp);
      }

      if let Some(attachment) = attachment {
        builder = builder.attachment(attachment);
      }

      builder.await.map_napi_err()
    })
  }

  #[napi]
  pub async fn matching_status(&self) -> napi::Result<MatchingStatus> {
    let m = self.get()?.matching_status().await.map_napi_err()?;
    Ok(m.into())
  }

  #[napi]
  pub async fn matching_listener(
    &self,
    options: Option<MatchingListenerOptions>,
  ) -> napi::Result<MatchingListener> {
    let MatchingListenerOptions { capacity } = options.unwrap_or_default();
    let (cb, receiver) = FifoChannel::with_capacity(capacity).into_handler();

    let listener = self
      .get()?
      .matching_listener()
      .with((cb, ()))
      .await
      .map_napi_err()?;

    Ok(MatchingListener::new(listener, receiver))
  }

  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    let publisher = self
      .inner
      .take()
      .ok_or_else(|| napi::Error::from_reason("publisher has already been undeclared"))?;

    env.spawn_future(async move {
      match Arc::into_inner(publisher) {
        Some(publisher) => publisher.undeclare().await.map_napi_err(),
        None => Ok(()),
      }
    })
  }
}
