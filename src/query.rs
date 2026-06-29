use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::{
  bytes as zbytes,
  query::{self as zquery},
  sample as zsample, time as ztime,
};

use crate::{
  bytes::*, config::*, encoding::*, error::*, key_expr::*, options::*, qos::*, sample::*,
  selector::*,
};

#[napi]
pub struct Query {
  inner: zquery::Query,
}

impl From<zquery::Query> for Query {
  fn from(inner: zquery::Query) -> Self {
    Query { inner }
  }
}

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
  pub fn reply<'env>(
    &self,
    env: &'env Env,
    key_expr: KeyExprArg,
    payload: PayloadArg,
    options: Option<ReplyOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, ()>> {
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
    let encoding = encoding.map(zbytes::Encoding::from);
    let timestamp = timestamp.map(|ts| ztime::Timestamp::from(ts.as_ref()));
    let attachment = attachment.map(IntoZBytes::into_zbytes);
    let source_info = source_info.map(|source_info| zsample::SourceInfo::from(&*source_info));

    env.spawn_future(async move {
      let mut builder = query.reply(expr, payload);
      if let Some(encoding) = encoding {
        builder = builder.encoding(encoding);
      }
      if let Some(express) = express {
        builder = builder.express(express);
      }
      if let Some(timestamp) = timestamp {
        builder = builder.timestamp(timestamp);
      }
      if let Some(attachment) = attachment {
        builder = builder.attachment(attachment);
      }
      if let Some(source_info) = source_info {
        builder = builder.source_info(source_info);
      }
      builder.await.map_napi_err()
    })
  }

  #[napi]
  pub async fn reply_err(
    &self,
    payload: PayloadArg,
    options: Option<ReplyErrOptions>,
  ) -> napi::Result<()> {
    let payload = payload.into_zbytes();

    let ReplyErrOptions { encoding } = options.unwrap_or_default();
    let encoding = encoding.map(zbytes::Encoding::from);

    let mut builder = self.inner.reply_err(payload);
    if let Some(encoding) = encoding {
      builder = builder.encoding(encoding);
    }
    builder.await.map_napi_err()
  }

  #[napi]
  pub fn reply_del<'env>(
    &self,
    env: &'env Env,
    key_expr: KeyExprArg<'_>,
    options: Option<ReplyDelOptions<'_>>,
  ) -> napi::Result<PromiseRaw<'env, ()>> {
    let expr = KeyExpr::try_from(key_expr)?;
    let query = self.inner.clone();

    let ReplyDelOptions {
      express,
      timestamp,
      attachment,
      source_info,
    } = options.unwrap_or_default();
    let timestamp = timestamp.map(|ts| ztime::Timestamp::from(ts.as_ref()));
    let attachment = attachment.map(IntoZBytes::into_zbytes);
    let source_info = source_info.map(|source_info| zsample::SourceInfo::from(&*source_info));

    env.spawn_future(async move {
      let mut builder = query.reply_del(expr);
      if let Some(express) = express {
        builder = builder.express(express);
      }
      if let Some(timestamp) = timestamp {
        builder = builder.timestamp(timestamp);
      }
      if let Some(attachment) = attachment {
        builder = builder.attachment(attachment);
      }
      if let Some(source_info) = source_info {
        builder = builder.source_info(source_info);
      }
      builder.await.map_napi_err()
    })
  }
}

#[napi(string_enum)]
pub enum ReplyKeyExpr {
  Any,
  MatchingQuery,
}

impl From<ReplyKeyExpr> for zquery::ReplyKeyExpr {
  fn from(value: ReplyKeyExpr) -> Self {
    match value {
      ReplyKeyExpr::Any => Self::Any,
      ReplyKeyExpr::MatchingQuery => Self::MatchingQuery,
    }
  }
}

impl From<zquery::ReplyKeyExpr> for ReplyKeyExpr {
  fn from(value: zquery::ReplyKeyExpr) -> Self {
    match value {
      zquery::ReplyKeyExpr::Any => Self::Any,
      zquery::ReplyKeyExpr::MatchingQuery => Self::MatchingQuery,
    }
  }
}

#[napi]
pub struct Reply {
  inner: zquery::Reply,
}

impl From<zquery::Reply> for Reply {
  fn from(inner: zquery::Reply) -> Self {
    Reply { inner }
  }
}

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

#[napi]
pub struct ReplyError {
  inner: zquery::ReplyError,
}

impl From<zquery::ReplyError> for ReplyError {
  fn from(inner: zquery::ReplyError) -> Self {
    ReplyError { inner }
  }
}

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

#[napi]
pub struct Parameters {
  inner: zquery::Parameters<'static>,
}

impl From<zquery::Parameters<'static>> for Parameters {
  fn from(inner: zquery::Parameters<'static>) -> Self {
    Parameters { inner }
  }
}

impl From<&Parameters> for zquery::Parameters<'static> {
  fn from(value: &Parameters) -> Self {
    value.inner.clone()
  }
}

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

#[napi(string_enum)]
pub enum QueryTarget {
  BestMatching,
  All,
  AllComplete,
}

impl From<QueryTarget> for zquery::QueryTarget {
  fn from(value: QueryTarget) -> Self {
    match value {
      QueryTarget::BestMatching => Self::BestMatching,
      QueryTarget::All => Self::All,
      QueryTarget::AllComplete => Self::AllComplete,
    }
  }
}

impl From<zquery::QueryTarget> for QueryTarget {
  fn from(value: zquery::QueryTarget) -> Self {
    match value {
      zquery::QueryTarget::BestMatching => Self::BestMatching,
      zquery::QueryTarget::All => Self::All,
      zquery::QueryTarget::AllComplete => Self::AllComplete,
    }
  }
}

#[napi(string_enum)]
pub enum ConsolidationMode {
  Auto,
  None,
  Monotonic,
  Latest,
}

impl From<ConsolidationMode> for zquery::ConsolidationMode {
  fn from(value: ConsolidationMode) -> Self {
    match value {
      ConsolidationMode::Auto => Self::Auto,
      ConsolidationMode::None => Self::None,
      ConsolidationMode::Monotonic => Self::Monotonic,
      ConsolidationMode::Latest => Self::Latest,
    }
  }
}

impl From<zquery::ConsolidationMode> for ConsolidationMode {
  fn from(value: zquery::ConsolidationMode) -> Self {
    match value {
      zquery::ConsolidationMode::Auto => Self::Auto,
      zquery::ConsolidationMode::None => Self::None,
      zquery::ConsolidationMode::Monotonic => Self::Monotonic,
      zquery::ConsolidationMode::Latest => Self::Latest,
    }
  }
}
