use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::query::{self as zquery};

use crate::{
  bytes::*, config::*, encoding::*, error::*, key_expr::*, macros::*, options::*, qos::*,
  sample::*, selector::*,
};

wrapper!(zquery::Query);

#[napi]
impl Query {
  #[napi(getter)]
  pub fn selector(&self) -> Selector {
    self.inner.selector().into_owned().into()
  }

  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    self.inner.key_expr().clone().into()
  }

  #[napi(getter)]
  pub fn payload(&self) -> Option<Bytes> {
    self.inner.payload().cloned().map(Bytes::from)
  }

  #[napi(getter)]
  pub fn encoding(&self) -> Option<Encoding> {
    self.inner.encoding().cloned().map(Into::into)
  }

  #[napi(getter)]
  pub fn attachment(&self) -> Option<Bytes> {
    self.inner.attachment().cloned().map(Bytes::from)
  }

  #[napi(getter)]
  pub fn source_info(&self) -> Option<SourceInfo> {
    self.inner.source_info().cloned().map(SourceInfo::from)
  }

  #[napi(getter)]
  pub fn priority(&self) -> Priority {
    self.inner.priority().into()
  }

  #[napi(getter)]
  pub fn congestion_control(&self) -> CongestionControl {
    self.inner.congestion_control().into()
  }

  #[napi(getter)]
  pub fn express(&self) -> bool {
    self.inner.express()
  }

  #[napi(getter)]
  pub fn parameters(&self) -> Parameters {
    self.inner.parameters().clone().into()
  }

  #[napi(getter)]
  pub fn accept_replies(&self) -> ReplyKeyExpr {
    self.inner.accepts_replies().into()
  }

  #[napi]
  pub async fn reply(
    &self,
    key_expr: KeyExprArg<'_>,
    payload: Payload,
    options: Option<ReplyOptions>,
  ) -> napi::Result<()> {
    let expr = KeyExpr::try_from(key_expr)?;
    let payload = payload.into_zbytes();
    let query = self.inner.clone();

    let ReplyOptions {
      encoding,
      express,
      timestamp,
      attachment,
      source_info,
    } = options.unwrap_or_default();

    build!(
      query.reply(expr, payload),
      encoding,
      express,
      timestamp,
      attachment,
      source_info,
    )
    .await
    .map_napi_err()
  }

  #[napi]
  pub async fn reply_err(
    &self,
    payload: Payload,
    options: Option<ReplyErrOptions>,
  ) -> napi::Result<()> {
    let payload = payload.into_zbytes();
    let ReplyErrOptions { encoding } = options.unwrap_or_default();

    build!(self.inner.reply_err(payload), encoding)
      .await
      .map_napi_err()
  }

  #[napi]
  pub async fn reply_del(
    &self,
    key_expr: KeyExprArg<'_>,
    options: Option<ReplyDelOptions>,
  ) -> napi::Result<()> {
    let expr = KeyExpr::try_from(key_expr)?;
    let query = self.inner.clone();

    let ReplyDelOptions {
      express,
      timestamp,
      attachment,
      source_info,
    } = options.unwrap_or_default();

    build!(
      query.reply_del(expr),
      express,
      timestamp,
      attachment,
      source_info,
    )
    .await
    .map_napi_err()
  }
}

enum_mapper!(zquery::ReplyKeyExpr: Any, MatchingQuery);

wrapper!(zquery::Reply);

#[napi]
impl Reply {
  #[napi(getter)]
  pub fn sample(&self) -> Option<Sample> {
    self.inner.result().ok().map(|s| Sample::new(s.clone()))
  }

  #[napi(getter)]
  pub fn error(&self) -> Option<ReplyError> {
    self.inner.result().err().map(|e| e.clone().into())
  }

  #[napi(getter)]
  pub fn replier_id(&self) -> Option<EntityGlobalId> {
    self.inner.replier_id().map(EntityGlobalId::from)
  }
}

#[allow(dead_code)]
#[napi(object)]
pub struct ReplySample<'env> {
  pub sample: ClassInstance<'env, Sample>,
  #[napi(ts_type = "null")]
  pub error: Option<ClassInstance<'env, ReplyError>>,
  #[napi(ts_type = "EntityGlobalId | null")]
  pub replier_id: ClassInstance<'env, EntityGlobalId>,
}

#[allow(dead_code)]
#[napi(object)]
pub struct ReplyErrored<'env> {
  #[napi(ts_type = "null")]
  pub sample: Option<ClassInstance<'env, Sample>>,
  pub error: ClassInstance<'env, ReplyError>,
  #[napi(ts_type = "EntityGlobalId | null")]
  pub replier_id: ClassInstance<'env, EntityGlobalId>,
}

#[allow(dead_code)]
#[napi]
pub type ReplyResult<'env> = Either<ReplySample<'env>, ReplyErrored<'env>>;

wrapper!(zquery::ReplyError);

#[napi]
impl ReplyError {
  #[napi(getter)]
  pub fn encoding(&self) -> Encoding {
    self.inner.encoding().clone().into()
  }

  #[napi(getter)]
  pub fn payload(&self) -> Bytes {
    self.inner.payload().clone().into()
  }
}

wrapper!(zquery::Parameters<'static>: Clone);

#[napi]
impl Parameters {
  #[napi(factory)]
  pub fn empty() -> Self {
    zquery::Parameters::empty().into()
  }

  #[napi(constructor)]
  pub fn new(params: String) -> Self {
    zquery::Parameters::from(params).into()
  }

  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.inner.as_str().to_string()
  }

  #[napi(getter)]
  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  #[napi(getter)]
  pub fn is_ordered(&self) -> bool {
    self.inner.is_ordered()
  }

  #[napi]
  pub fn contains_key(&self, key: String) -> bool {
    self.inner.contains_key(key)
  }

  #[napi]
  pub fn get(&self, key: String) -> Option<String> {
    self.inner.get(key).map(|value| value.to_string())
  }

  #[napi]
  pub fn values(&self, key: String) -> Vec<String> {
    self
      .inner
      .values(key)
      .map(|value| value.to_string())
      .collect()
  }

  #[napi]
  pub fn insert(&mut self, key: String, value: String) -> Option<String> {
    self.inner.insert(key, value)
  }

  #[napi]
  pub fn remove(&mut self, key: String) -> Option<String> {
    self.inner.remove(key)
  }

  #[napi]
  pub fn extend(&mut self, other: &Parameters) {
    self.inner.extend(&other.inner);
  }
}

enum_mapper!(zquery::QueryTarget: BestMatching, All, AllComplete);

enum_mapper!(zquery::ConsolidationMode: Auto, None, Monotonic, Latest);
