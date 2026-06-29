use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{
  bytes as zbytes, cancellation as zcancellation, key_expr as zkey_expr, matching as zmatching,
  qos as zqos, query as zquery, sample as zsample, session as zsession,
};

use crate::{
  bytes::*, config::*, error::*, key_expr::*, matching::*, options::*, qos::*, query::*,
};

#[napi]
pub struct Querier {
  inner: zquery::Querier<'static>,
}

impl From<zquery::Querier<'static>> for Querier {
  fn from(querier: zquery::Querier<'static>) -> Self {
    Self { inner: querier }
  }
}

#[napi]
impl Querier {
  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    self.inner.key_expr().clone().into()
  }
  #[napi(getter)]
  pub fn id(&self) -> EntityGlobalId {
    self.inner.id().into()
  }

  #[napi(getter)]
  pub fn congestion_control(&self) -> CongestionControl {
    self.inner.congestion_control().into()
  }

  #[napi(getter)]
  pub fn priority(&self) -> Priority {
    self.inner.priority().into()
  }

  #[napi(getter)]
  pub fn accept_replies(&self) -> ReplyKeyExpr {
    self.inner.accept_replies().into()
  }

  #[napi]
  pub fn get<'env>(
    &self,
    env: &'env Env,
    options: Option<QuerierGetOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, Either<(), ()>>> {
    todo!()

    // let QuerierGetOptions {
    //   parameters,
    //   payload,
    //   encoding,
    //   attachment,
    //   source_info,
    //   cancellation_token,
    //   channel,
    // } = options.unwrap_or_default();
    // let parameters = parameters.map(|p| zquery::Parameters::from(p.as_ref()));
    // let payload = payload.map(IntoZBytes::into_zbytes);
    // let encoding = encoding.map(zbytes::Encoding::from);
    // let attachment = attachment.map(IntoZBytes::into_zbytes);
    // let source_info = source_info.map(|si| zsample::SourceInfo::from(si.as_ref()));
    // let cancellation_token =
    //   cancellation_token.map(|ct| zcancellation::CancellationToken::from(&*ct));
    // let querier = self.inner;

    // env.spawn_future(async move {
    //   let mut builder = querier.get().with(callback);
    //   if let Some(parameters) = parameters {
    //     builder = builder.parameters(parameters);
    //   }
    //   if let Some(payload) = payload {
    //     builder = builder.payload(payload);
    //   }
    //   if let Some(encoding) = encoding {
    //     builder = builder.encoding(encoding);
    //   }
    //   if let Some(attachment) = attachment {
    //     builder = builder.attachment(attachment);
    //   }
    //   if let Some(source_info) = source_info {
    //     builder = builder.source_info(source_info);
    //   }
    //   if let Some(cancellation_token) = cancellation_token {
    //     builder = builder.cancellation_token(cancellation_token);
    //   }
    //   builder.await.map_napi_err()?;
    //   Ok(receiver.handler())
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
    // let (callback, receiver) = ChannelReceiver::<zmatching::MatchingStatus>::split(channel);
    // let querier = Arc::clone(self.inner.get()?);

    // env.spawn_future(async move {
    //   let listener = querier
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
