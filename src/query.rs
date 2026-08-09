use std::collections::HashMap;

use derive_more::{AsRef, From, Into};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh as z;

use crate::{
  bytes::*, config::*, handlers::*, key_expr::*, matching::*, options::*, qos::*, sample::*,
  utils::*,
};

#[derive(From)]
#[from(forward)]
#[napi]
pub struct Query(Declared<z::query::Query>);

#[napi]
impl Query {
  #[napi(getter)]
  pub fn selector(&self) -> napi::Result<Selector> {
    Ok(self.0.get()?.selector().into_owned().into())
  }

  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.0.get()?.key_expr().clone().into())
  }

  #[napi(getter)]
  pub fn payload(&self) -> napi::Result<Option<Bytes>> {
    Ok(self.0.get()?.payload().cloned().map(Into::into))
  }

  #[napi(getter)]
  pub fn encoding(&self) -> napi::Result<Option<Encoding>> {
    Ok(self.0.get()?.encoding().cloned().map(Into::into))
  }

  #[napi(getter)]
  pub fn attachment(&self) -> napi::Result<Option<Bytes>> {
    Ok(self.0.get()?.attachment().cloned().map(Into::into))
  }

  #[napi(getter)]
  pub fn source_info(&self) -> napi::Result<Option<SourceInfo>> {
    Ok(self.0.get()?.source_info().cloned().map(Into::into))
  }

  #[napi(getter)]
  pub fn priority(&self) -> napi::Result<Priority> {
    Ok(self.0.get()?.priority().into())
  }

  #[napi(getter)]
  pub fn congestion_control(&self) -> napi::Result<CongestionControl> {
    Ok(self.0.get()?.congestion_control().into())
  }

  #[napi(getter)]
  pub fn express(&self) -> napi::Result<bool> {
    Ok(self.0.get()?.express())
  }

  #[napi(getter)]
  pub fn parameters(&self) -> napi::Result<Parameters<'_>> {
    Ok(self.0.get()?.parameters().clone().into())
  }

  #[napi(getter)]
  pub fn accepts_replies(&self) -> napi::Result<ReplyKeyExpr> {
    Ok(self.0.get()?.accepts_replies().into())
  }

  #[napi]
  pub async fn reply(
    &self,
    key_expr: KeyExprArg<'_>,
    payload: BytesBuffer,
    options: Option<ReplyOptions>,
  ) -> napi::Result<()> {
    let expr = KeyExpr::try_from(key_expr)?;
    let query = self.0.get()?;
    let ReplyOptions {
      encoding,
      express,
      attachment,
    } = options.unwrap_or_default();
    let mut builder = query.reply(expr, payload);

    if let Some(encoding) = encoding {
      builder = builder.encoding(encoding)
    }

    if let Some(express) = express {
      builder = builder.express(express)
    }

    if let Some(attachment) = attachment {
      builder = builder.attachment(attachment)
    }

    builder.await.map_napi_err()
  }

  #[napi]
  pub async fn reply_err(
    &self,
    payload: BytesBuffer,
    options: Option<ReplyErrOptions>,
  ) -> napi::Result<()> {
    let ReplyErrOptions { encoding } = options.unwrap_or_default();
    let mut builder = self.0.get()?.reply_err(payload);
    if let Some(e) = encoding {
      builder = builder.encoding(e);
    }

    builder.await.map_napi_err()
  }

  #[napi]
  pub async fn reply_del(
    &self,
    key_expr: KeyExprArg<'_>,
    options: Option<ReplyDelOptions>,
  ) -> napi::Result<()> {
    let expr = KeyExpr::try_from(key_expr)?;
    let ReplyDelOptions {
      express,
      attachment,
    } = options.unwrap_or_default();
    let mut builder = self.0.get()?.reply_del(expr);

    if let Some(express) = express {
      builder = builder.express(express)
    }

    if let Some(attachment) = attachment {
      builder = builder.attachment(attachment)
    }

    builder.await.map_napi_err()
  }

  #[napi]
  pub fn drop(&mut self) -> napi::Result<()> {
    self.0.take()?;
    Ok(())
  }
}

#[napi(string_enum)]
pub enum ReplyKeyExpr {
  Any,
  MatchingQuery,
}

impl From<ReplyKeyExpr> for z::query::ReplyKeyExpr {
  fn from(value: ReplyKeyExpr) -> Self {
    match value {
      ReplyKeyExpr::Any => Self::Any,
      ReplyKeyExpr::MatchingQuery => Self::MatchingQuery,
    }
  }
}

impl From<z::query::ReplyKeyExpr> for ReplyKeyExpr {
  fn from(value: z::query::ReplyKeyExpr) -> Self {
    match value {
      z::query::ReplyKeyExpr::Any => Self::Any,
      z::query::ReplyKeyExpr::MatchingQuery => Self::MatchingQuery,
    }
  }
}

#[derive(From, Into)]
#[napi]
pub struct Reply(z::query::Reply);

#[napi]
impl Reply {
  #[napi(getter)]
  pub fn replier_id(&self) -> Option<EntityGlobalId> {
    self.0.replier_id().map(EntityGlobalId::from)
  }

  #[napi(getter)]
  pub fn result<'env>(&self, env: &'env Env) -> napi::Result<ReplyResult<'env>> {
    match self.0.result() {
      Ok(sample) => Ok(Either::A(ReplyResultSample {
        sample: Sample::from(sample.clone()).into_instance(env)?,
        error: Null,
      })),
      Err(error) => Ok(Either::B(ReplyResultError {
        sample: Null,
        error: ReplyError::from(error.clone()).into_instance(env)?,
      })),
    }
  }
}

#[napi(object)]
pub struct ReplyResultSample<'env> {
  pub sample: ClassInstance<'env, Sample>,
  pub error: Null,
}

#[napi(object)]
pub struct ReplyResultError<'env> {
  pub sample: Null,
  pub error: ClassInstance<'env, ReplyError>,
}

#[napi]
pub type ReplyResult<'env> = Either<ReplyResultSample<'env>, ReplyResultError<'env>>;

#[derive(From, Into)]
#[napi]
pub struct ReplyError(z::query::ReplyError);

#[napi]
impl ReplyError {
  #[napi(getter)]
  pub fn encoding(&self) -> Encoding {
    self.0.encoding().clone().into()
  }

  #[napi(getter)]
  pub fn payload(&self) -> Bytes {
    self.0.payload().clone().into()
  }
}

#[napi]
pub type ParametersLike = HashMap<String, String>;

impl<'s> From<ParametersLike> for Parameters<'s> {
  fn from(value: ParametersLike) -> Self {
    Parameters::from(value)
  }
}

#[derive(Clone, From, Into)]
#[napi]
pub struct Parameters<'s>(z::query::Parameters<'s>);

#[napi]
impl<'s> Parameters<'s> {
  #[napi(factory)]
  pub fn empty() -> Self {
    z::query::Parameters::empty().into()
  }

  #[napi(constructor)]
  pub fn new(params: String) -> Self {
    z::query::Parameters::from(params).into()
  }

  #[napi(factory)]
  pub fn from(params: ParametersLike) -> Self {
    z::query::Parameters::from(params).into()
  }

  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.0.as_str().to_string()
  }

  #[napi(getter)]
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  #[napi(getter)]
  pub fn is_ordered(&self) -> bool {
    self.0.is_ordered()
  }

  #[napi]
  pub fn contains_key(&self, key: String) -> bool {
    self.0.contains_key(key)
  }

  #[napi]
  pub fn get(&self, key: String) -> Option<String> {
    self.0.get(key).map(|value| value.to_string())
  }

  #[napi]
  pub fn values(&self, key: String) -> Vec<String> {
    self.0.values(key).map(|value| value.to_string()).collect()
  }

  #[napi]
  pub fn insert(&mut self, key: String, value: String) -> Option<String> {
    self.0.insert(key, value)
  }

  #[napi]
  pub fn remove(&mut self, key: String) -> Option<String> {
    self.0.remove(key)
  }

  #[napi]
  pub fn extend(&mut self, other: &Parameters) {
    self.0.extend(&other.0);
  }
}

#[napi(string_enum)]
pub enum QueryTarget {
  BestMatching,
  All,
  AllComplete,
}

impl From<QueryTarget> for z::query::QueryTarget {
  fn from(value: QueryTarget) -> Self {
    match value {
      QueryTarget::BestMatching => Self::BestMatching,
      QueryTarget::All => Self::All,
      QueryTarget::AllComplete => Self::AllComplete,
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

impl From<ConsolidationMode> for z::query::ConsolidationMode {
  fn from(value: ConsolidationMode) -> Self {
    match value {
      ConsolidationMode::Auto => Self::Auto,
      ConsolidationMode::None => Self::None,
      ConsolidationMode::Monotonic => Self::Monotonic,
      ConsolidationMode::Latest => Self::Latest,
    }
  }
}

impl From<ConsolidationMode> for z::query::QueryConsolidation {
  fn from(value: ConsolidationMode) -> Self {
    let mode: z::query::ConsolidationMode = value.into();
    mode.into()
  }
}

#[derive(From)]
#[from(forward)]
#[napi]
pub struct Queryable(Declared<z::query::Queryable<HandlerImpl<z::query::Query>>>);

#[napi]
impl Queryable {
  #[napi(getter)]
  pub fn id(&self) -> napi::Result<EntityGlobalId> {
    Ok(self.0.get()?.id().into())
  }

  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.0.get()?.key_expr().clone().into())
  }

  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    z::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }

  #[napi]
  pub async fn recv(&self) -> napi::Result<DeferredJs> {
    self.0.get()?.recv().await
  }

  #[napi]
  pub fn try_recv(&self) -> napi::Result<Option<DeferredJs>> {
    self.0.get()?.try_recv()
  }

  #[napi]
  pub fn stream(&self) -> napi::Result<Stream> {
    Ok(self.0.get()?.stream())
  }

  #[napi]
  pub fn handler(&self) -> napi::Result<Handler> {
    Ok(self.0.get()?.share())
  }
}

#[derive(From)]
#[from(forward)]
#[napi]
pub struct Querier(Declared<z::query::Querier<'static>>);

#[napi]
impl Querier {
  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.0.get()?.key_expr().clone().into())
  }

  #[napi(getter)]
  pub fn id(&self) -> napi::Result<EntityGlobalId> {
    Ok(self.0.get()?.id().into())
  }

  #[napi(getter)]
  pub fn congestion_control(&self) -> napi::Result<CongestionControl> {
    Ok(self.0.get()?.congestion_control().into())
  }

  #[napi(getter)]
  pub fn priority(&self) -> napi::Result<Priority> {
    Ok(self.0.get()?.priority().into())
  }

  #[napi(getter)]
  pub fn accept_replies(&self) -> napi::Result<ReplyKeyExpr> {
    Ok(self.0.get()?.accept_replies().into())
  }

  #[napi]
  pub async fn get(&self, options: Option<QuerierGetOptions>) -> napi::Result<Handler> {
    let QuerierGetOptions {
      parameters,
      payload,
      encoding,
      attachment,
      // cancellation_token,
      channel,
    } = options.unwrap_or_default();
    let handler = into_handler(channel);
    let mut builder = self.0.get()?.get().with(handler);

    if let Some(parameters) = parameters {
      builder = builder.parameters(Parameters::from(parameters))
    }

    if let Some(payload) = payload {
      builder = builder.payload(payload)
    }

    if let Some(encoding) = encoding {
      builder = builder.encoding(encoding)
    }

    if let Some(attachment) = attachment {
      builder = builder.attachment(attachment)
    }

    let receiver = builder.await.map_napi_err()?;

    Ok(receiver.into())
  }

  #[napi]
  pub async fn matching_status(&self) -> napi::Result<MatchingStatus> {
    let status = self.0.get()?.matching_status().await.map_napi_err()?;

    Ok(status.into())
  }

  #[napi]
  pub async fn matching_listener(
    &self,
    options: Option<MatchingListenerOptions>,
  ) -> napi::Result<MatchingListener> {
    let MatchingListenerOptions { channel } = options.unwrap_or_default();
    let handler = into_handler(channel);

    let listener = self
      .0
      .get()?
      .matching_listener()
      .with(handler)
      .await
      .map_napi_err()?;

    Ok(listener.into())
  }

  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    z::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }
}

#[derive(AsRef, From, Into)]
#[napi]
pub struct Selector(z::query::Selector<'static>);

impl Selector {
  pub(crate) fn resolve(
    selector: SelectorArg<'_>,
    parameters: Option<ParametersLike>,
  ) -> napi::Result<z::query::Selector<'static>> {
    let mut selector = z::query::Selector::from(Selector::try_from(selector)?);
    if let Some(parameters) = parameters {
      let parameters = z::query::Parameters::from(parameters);
      let key_expr = selector.key_expr().clone().into_owned();
      selector = z::query::Selector::owned(key_expr, parameters);
    }
    Ok(selector)
  }
}

#[napi(object)]
pub struct SelectorParts {
  pub key_expr: String,
  pub parameters: String,
}

#[napi]
impl Selector {
  #[napi(constructor)]
  pub fn new(key_expr: KeyExprArg, parameters: Option<String>) -> napi::Result<Self> {
    let key_expr = KeyExpr::try_from(key_expr)?;

    let parameters = parameters
      .map(z::query::Parameters::from)
      .unwrap_or_default();

    let inner = z::query::Selector::owned(key_expr, parameters);

    Ok(inner.into())
  }

  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    self.0.key_expr().clone().into_owned().into()
  }

  #[napi(getter)]
  pub fn parameters(&self) -> Parameters<'_> {
    self.0.parameters().clone().into_owned().into()
  }

  #[napi]
  pub fn split(&self) -> SelectorParts {
    SelectorParts {
      key_expr: self.0.key_expr().as_str().to_string(),
      parameters: self.0.parameters().as_str().to_string(),
    }
  }
}

#[napi]
pub type SelectorArg<'a> = Either3<String, &'a KeyExpr, &'a Selector>;

impl TryFrom<SelectorArg<'_>> for Selector {
  type Error = napi::Error;

  fn try_from(value: SelectorArg<'_>) -> napi::Result<Self> {
    match value {
      Either3::A(selector) => z::query::Selector::try_from(selector)
        .map(Selector::from)
        .map_napi_err(),
      Either3::B(key_expr) => {
        Ok(z::query::Selector::from(zenoh::key_expr::KeyExpr::from(key_expr.clone())).into())
      }
      Either3::C(selector) => Ok(selector.as_ref().clone().into()),
    }
  }
}
