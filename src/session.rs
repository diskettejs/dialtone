use std::time::Duration;

use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{cancellation as zcancellation, query as zquery, sample as zsample};
use zenoh_ext::{AdvancedPublisherBuilderExt, AdvancedSubscriberBuilderExt};

use crate::{
  bytes::*, config::*, error::*, info::*, key_expr::*, liveliness::*, options::*, publisher::*,
  querier::*, queryable::*, selector::*, subscriber::*, time::*,
};

#[napi]
pub struct Session {
  inner: zenoh::Session,
}

impl From<zenoh::Session> for Session {
  fn from(inner: zenoh::Session) -> Self {
    Session { inner }
  }
}

#[napi]
impl Session {
  #[napi(factory)]
  pub async fn open(config: &Config) -> napi::Result<Self> {
    let cfg = zenoh::config::Config::from(config);
    let session = zenoh::open(cfg).await.map_napi_err()?;
    Ok(session.into())
  }

  #[napi(getter)]
  pub fn zid(&self) -> String {
    self.inner.zid().to_string()
  }

  #[napi(getter)]
  pub fn id(&self) -> EntityGlobalId {
    self.inner.id().into()
  }

  #[napi(getter)]
  pub fn is_closed(&self) -> bool {
    self.inner.is_closed()
  }

  #[napi]
  pub fn new_timestamp(&self) -> Timestamp {
    self.inner.new_timestamp().into()
  }

  #[napi]
  pub fn info(&self) -> SessionInfo {
    self.inner.info().into()
  }

  #[napi]
  pub fn config(&self) -> SessionConfig {
    self.inner.config().into()
  }

  #[napi]
  pub async fn close(&self) -> napi::Result<()> {
    self.inner.close().await.map_napi_err()
  }

  #[napi]
  pub fn put<'env>(
    &self,
    env: &'env Env,
    key_expr: KeyExprArg,
    payload: PayloadArg,
    options: Option<PutOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, ()>> {
    todo!()

    // let expr = KeyExpr::try_from(key_expr)?;
    // let opts = options.unwrap_or_default();
    // let payload = payload.into_zbytes();
    // let session = self.inner.clone();
    // TODO: map options to build session builder calls
    // env.spawn_future(async move { session.put(expr, payload).await.map_napi_err() })
  }

  #[napi]
  pub fn get<'env>(
    &self,
    env: &'env Env,
    selector: SelectorArg<'_>,
    options: Option<GetOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, Either<(), ()>>> {
    todo!()

    // let GetOptions {
    //   parameters,
    //   target,
    //   consolidation,
    //   congestion_control,
    //   priority,
    //   express,
    //   allowed_destination,
    //   timeout,
    //   payload,
    //   encoding,
    //   attachment,
    //   source_info,
    //   cancellation_token,
    //   channel,
    // } = options.unwrap_or_default();

    // let mut selector = zquery::Selector::from(Selector::try_from(selector)?);
    // if let Some(parameters) = parameters {
    //   let key_expr = selector.key_expr().clone().into_owned();
    //   selector = zquery::Selector::owned(key_expr, zquery::Parameters::from(&*parameters));
    // }

    // let timeout = timeout
    //   .map(|ms| Duration::try_from_secs_f64(ms / 1000.0).map_napi_err())
    //   .transpose()?;
    // let payload = payload.map(|payload| payload.into_zbytes());
    // let attachment = attachment.map(|attachment| attachment.into_zbytes());
    // let source_info = source_info.map(|source_info| zsample::SourceInfo::from(&*source_info));
    // let cancellation_token =
    //   cancellation_token.map(|ct| zcancellation::CancellationToken::from(&*ct));
    // let session = self.inner.clone();

    // env.spawn_future(async move {
    //   let mut builder = session.get(selector).with(callback);

    //   if let Some(target) = target {
    //     builder = builder.target(target.into());
    //   }

    //   if let Some(consolidation) = consolidation {
    //     builder = builder.consolidation(zquery::ConsolidationMode::from(consolidation));
    //   }

    //   if let Some(congestion_control) = congestion_control {
    //     builder = builder.congestion_control(congestion_control.into());
    //   }

    //   if let Some(priority) = priority {
    //     builder = builder.priority(priority.into());
    //   }

    //   if let Some(express) = express {
    //     builder = builder.express(express);
    //   }

    //   if let Some(allowed_destination) = allowed_destination {
    //     builder = builder.allowed_destination(allowed_destination.into());
    //   }

    //   if let Some(timeout) = timeout {
    //     builder = builder.timeout(timeout);
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
  pub fn delete<'env>(
    &self,
    env: &'env Env,
    key_expr: KeyExprArg,
    options: Option<DeleteOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, ()>> {
    todo!()

    // let expr = KeyExpr::try_from(key_expr)?;
    // let session = self.inner.clone();
    // // TODO: map options to build session builder calls
    // env.spawn_future(async move { session.delete(expr).await.map_napi_err() })
  }

  #[napi]
  pub fn liveliness(&self) -> Liveliness {
    todo!()

    // self.inner.clone().into()
  }

  #[napi]
  pub async fn declare_keyexpr(&self, key_expr: KeyExprArg<'_>) -> napi::Result<KeyExpr> {
    todo!()

    // let key_expr = KeyExpr::try_from(key_expr)?;
    // let declared = self.inner.declare_keyexpr(key_expr).await.map_napi_err()?;

    // Ok(declared.into())
  }

  #[napi]
  pub async fn declare_querier(
    &self,
    key_expr: KeyExprArg<'_>,
    options: Option<QuerierOptions>,
  ) -> napi::Result<Querier> {
    todo!()

    // let QuerierOptions {
    //   target,
    //   consolidation,
    //   congestion_control,
    //   priority,
    //   express,
    //   allowed_destination,
    //   timeout,
    //   accept_replies,
    // } = options.unwrap_or_default();
    // let expr = KeyExpr::try_from(key_expr)?;
    // let mut builder = self.inner.declare_querier(expr);

    // if let Some(target) = target {
    //   builder = builder.target(target.into());
    // }

    // if let Some(consolidation) = consolidation {
    //   builder = builder.consolidation(zquery::ConsolidationMode::from(consolidation));
    // }

    // if let Some(congestion_control) = congestion_control {
    //   builder = builder.congestion_control(congestion_control.into());
    // }

    // if let Some(priority) = priority {
    //   builder = builder.priority(priority.into());
    // }

    // if let Some(express) = express {
    //   builder = builder.express(express);
    // }

    // if let Some(allowed_destination) = allowed_destination {
    //   builder = builder.allowed_destination(allowed_destination.into());
    // }

    // if let Some(timeout) = timeout {
    //   builder = builder.timeout(Duration::try_from_secs_f64(timeout / 1000.0).map_napi_err()?);
    // }

    // if let Some(accept_replies) = accept_replies {
    //   builder = builder.accept_replies(accept_replies.into());
    // }

    // let zquerier = builder.await.map_napi_err()?;

    // Ok(zquerier.into())
  }

  #[napi]
  pub fn declare_queryable<'env>(
    &self,
    env: &'env Env,
    key_expr: KeyExprArg<'_>,
    options: Option<QueryableOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, Queryable>> {
    todo!()

    // let QueryableOptions {
    //   channel,
    //   allowed_origin,
    //   complete,
    // } = options.unwrap_or_default();
    // let expr = KeyExpr::try_from(key_expr)?;
    // let session = self.inner.clone();

    // env.spawn_future(async move {
    //   let mut builder = session.declare_queryable(expr).with(zenoh_channel);

    //   if let Some(origin) = allowed_origin {
    //     builder = builder.allowed_origin(origin.into());
    //   };

    //   if let Some(complete) = complete {
    //     builder = builder.complete(complete);
    //   }

    //   let inner = builder.await.map_napi_err()?;

    //   Ok(Queryable::new(inner, receiver))
    // })
  }

  #[napi]
  pub fn declare_subscriber<'env>(
    &self,
    env: &'env Env,
    key_expr: KeyExprArg,
    options: Option<SubscriberOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, Subscriber>> {
    let key_expr = KeyExpr::try_from(key_expr)?;
    let SubscriberOptions {
      allowed_origin,
      history,
      query_timeout_ms,
      recovery,
      subscriber_detection,
      subscriber_detection_metadata,
      channel,
    } = options.unwrap_or_default();
    let session = self.inner.clone();

    // env.spawn_future(async move {
    //   let subscriber = session
    //     .declare_subscriber(key_expr)
    //     .advanced()
    //     .with(zenoh_channel)
    //     .await
    //     .map_napi_err()?;
    //   Ok(subscriber.into())
    // });

    todo!()
  }

  #[napi]
  pub async fn declare_publisher(
    &self,
    key_expr: KeyExprArg<'_>,
    options: Option<PublisherOptions>,
  ) -> napi::Result<Publisher> {
    let expr = KeyExpr::try_from(key_expr)?;
    let PublisherOptions {
      cache,
      encoding,
      congestion_control,
      express,
      allowed_destination,
      priority,
      publisher_detection,
      publisher_detection_metadata,
      reliability,
      sample_miss_detection,
    } = options.unwrap_or_default();
    // let mut builder = self.inner.declare_publisher(expr).advanced();

    // if let Some(sample_miss_detection) = options.and_then(|options| options.sample_miss_detection) {
    //   builder = builder.sample_miss_detection(sample_miss_detection.into());
    // }

    // let zpublisher = builder.await.map_napi_err()?;

    // Ok(Publisher::from(zpublisher))

    todo!()
  }
}
