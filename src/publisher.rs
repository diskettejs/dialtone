use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{
  bytes as zbytes,
  internal::traits::{EncodingBuilderTrait, TimestampBuilderTrait},
  key_expr as zkey_expr, matching as zmatching, qos as zqos, session as zsession, time as ztime,
};

use crate::{
  bytes::*, config::*, encoding::*, error::*, key_expr::*, matching::*, options::*, qos::*,
};

#[napi]
pub struct Publisher {
  inner: zenoh_ext::AdvancedPublisher<'static>,
}

impl From<zenoh_ext::AdvancedPublisher<'static>> for Publisher {
  fn from(inner: zenoh_ext::AdvancedPublisher<'static>) -> Self {
    Self { inner }
  }
}

#[napi]
impl Publisher {
  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    self.inner.key_expr().clone().into()
  }

  #[napi(getter)]
  pub fn id(&self) -> EntityGlobalId {
    self.inner.id().into()
  }

  #[napi(getter)]
  pub fn encoding(&self) -> Encoding {
    self.inner.encoding().clone().into()
  }

  #[napi(getter)]
  pub fn congestion_control(&self) -> CongestionControl {
    self.inner.congestion_control().into()
  }

  #[napi(getter)]
  pub fn priority(&self) -> Priority {
    self.inner.priority().into()
  }

  #[napi]
  pub fn put<'env>(
    &self,
    env: &'env Env,
    payload: PayloadArg,
    options: Option<PublisherPutOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, ()>> {
    todo!()
    // let payload = payload.into_zbytes();

    // let PublisherPutOptions {
    //   encoding,
    //   timestamp,
    //   attachment,
    // } = options.unwrap_or_default();
    // let encoding = encoding.map(zbytes::Encoding::from);
    // let timestamp = timestamp.map(|ts| ztime::Timestamp::from(ts.as_ref()));
    // let attachment = attachment.map(IntoZBytes::into_zbytes);

    // let publisher = Arc::clone(self.inner.get()?);
    // env.spawn_future(async move {
    //   let mut builder = publisher.put(payload);
    //   if let Some(encoding) = encoding {
    //     builder = builder.encoding(encoding);
    //   }
    //   if let Some(timestamp) = timestamp {
    //     builder = builder.timestamp(timestamp);
    //   }
    //   if let Some(attachment) = attachment {
    //     builder = builder.attachment(attachment);
    //   }
    //   builder.await.map_napi_err()
    // })
  }

  #[napi]
  pub fn delete<'env>(
    &self,
    env: &'env Env,
    options: Option<PublisherDeleteOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, ()>> {
    todo!()
    // let PublisherDeleteOptions {
    //   timestamp,
    //   attachment,
    // } = options.unwrap_or_default();
    // let timestamp = timestamp.map(|ts| ztime::Timestamp::from(ts.as_ref()));
    // let attachment = attachment.map(IntoZBytes::into_zbytes);

    // env.spawn_future(async move {
    //   let mut builder = publisher.delete();
    //   if let Some(timestamp) = timestamp {
    //     builder = builder.timestamp(timestamp);
    //   }
    //   if let Some(attachment) = attachment {
    //     builder = builder.attachment(attachment);
    //   }
    //   builder.await.map_napi_err()
    // })
  }

  #[napi]
  pub async fn matching_status(&self) -> napi::Result<MatchingStatus> {
    todo!()

    // let m = self.inner.get()?.matching_status().await.map_napi_err()?;
    // Ok(m.into())
  }

  #[napi]
  pub fn matching_listener<'env>(
    &self,
    env: &'env Env,
    options: Option<MatchingListenerOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, MatchingListener>> {
    todo!()
    // let MatchingListenerOptions { channel } = options.unwrap_or_default();

    // env.spawn_future(async move {
    //   let listener = publisher
    //     .matching_listener()
    //     .with(callback)
    //     .await
    //     .map_napi_err()?;

    //   Ok(MatchingListener::new(listener, receiver))
    // })
  }

  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    todo!()
  }
}
