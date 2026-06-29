use std::{alloc::System, time::SystemTime};

use chrono::Utc;
use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{
  bytes as zbytes,
  internal::traits::*,
  query::{self as zquery},
  sample as zsample, time as ztime,
};

use crate::{
  bytes::*, config::*, encoding::*, error::MapNapiErr, key_expr::*, options::*, qos::*, sample::*,
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
  #[napi]
  pub fn result(&self) -> Either<Sample, ReplyError> {
    match self.inner.result() {
      Ok(s) => Either::A(Sample::new(s.clone())),
      Err(e) => Either::B(e.clone().into()),
    }
  }

  #[napi(getter)]
  pub fn replier_id(&self) -> Option<EntityGlobalId> {
    self.inner.replier_id().map(EntityGlobalId::from)
  }
}

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

#[napi]
pub struct TimeRange {
  inner: zquery::TimeRange,
}

impl From<zquery::TimeRange> for TimeRange {
  fn from(inner: zquery::TimeRange) -> Self {
    Self { inner }
  }
}

#[napi]
impl TimeRange {
  pub fn resolve_at(&self, now: chrono::DateTime<Utc>) -> TimeRange {
    // self.inner.resolve_at(now.into())
    todo!()
  }

  pub fn resolve(self) -> TimeRange {
    todo!()
  }
  pub fn contains(&self, instant: SystemTime) -> bool {
    todo!()
  }
}
