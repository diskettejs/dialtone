use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::{
  Wait,
  query::{self as zquery},
};

use crate::{
  bytes::*, config::*, handlers::*, key_expr::*, macros::*, matching::*, options::*, qos::*,
  sample::*, utils::*,
};

option_wrapper!(zquery::Query, "Dropped query");

#[napi]
impl Query {
  #[napi(getter)]
  pub fn selector(&self) -> napi::Result<Selector> {
    Ok(self.get_ref()?.selector().into_owned().into())
  }

  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.get_ref()?.key_expr().clone().into())
  }

  #[napi(getter)]
  pub fn payload(&self) -> napi::Result<Option<Bytes>> {
    Ok(self.get_ref()?.payload().cloned().map(Bytes::from))
  }

  #[napi(getter)]
  pub fn encoding(&self) -> napi::Result<Option<Encoding>> {
    Ok(self.get_ref()?.encoding().cloned().map(Into::into))
  }

  #[napi(getter)]
  pub fn attachment(&self) -> napi::Result<Option<Bytes>> {
    Ok(self.get_ref()?.attachment().cloned().map(Bytes::from))
  }

  #[napi(getter)]
  pub fn source_info(&self) -> napi::Result<Option<SourceInfo>> {
    Ok(self.get_ref()?.source_info().cloned().map(SourceInfo::from))
  }

  #[napi(getter)]
  pub fn priority(&self) -> napi::Result<Priority> {
    Ok(self.get_ref()?.priority().into())
  }

  #[napi(getter)]
  pub fn congestion_control(&self) -> napi::Result<CongestionControl> {
    Ok(self.get_ref()?.congestion_control().into())
  }

  #[napi(getter)]
  pub fn express(&self) -> napi::Result<bool> {
    Ok(self.get_ref()?.express())
  }

  #[napi(getter)]
  pub fn parameters(&self) -> napi::Result<Parameters> {
    Ok(self.get_ref()?.parameters().clone().into())
  }

  #[napi(getter)]
  pub fn accept_replies(&self) -> napi::Result<ReplyKeyExpr> {
    Ok(self.get_ref()?.accepts_replies().into())
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
    let query = self.get_ref()?.clone();

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

    build!(self.get_ref()?.reply_err(payload), encoding)
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
    let query = self.get_ref()?.clone();

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

  #[napi]
  pub fn drop(&mut self) -> napi::Result<()> {
    self.take()?;
    Ok(())
  }
}

enum_mapper!(zquery::ReplyKeyExpr: Any, MatchingQuery);

wrapper!(zquery::Reply);

#[napi]
impl Reply {
  #[napi(getter)]
  pub fn sample(&self) -> Option<Sample> {
    self.inner.result().ok().map(|s| Sample::from(s.clone()))
  }

  #[napi(getter)]
  pub fn error(&self) -> Option<ReplyError> {
    self.inner.result().err().map(|e| e.clone().into())
  }

  #[napi(getter)]
  pub fn replier_id(&self) -> Option<EntityGlobalId> {
    self.inner.replier_id().map(EntityGlobalId::from)
  }

  #[napi(getter)]
  pub fn is_sample(&self) -> bool {
    self.inner.result().is_ok()
  }

  #[napi(getter)]
  pub fn is_error(&self) -> bool {
    self.inner.result().is_err()
  }

  #[napi(getter, ts_return_type = "ReplyResult")]
  pub fn result(&self) -> Reply {
    self.inner.clone().into()
  }
}

#[allow(dead_code)]
#[napi(object)]
pub struct ReplySample<'env> {
  #[napi(ts_type = "true")]
  pub is_sample: bool,
  #[napi(ts_type = "false")]
  pub is_error: bool,
  pub sample: ClassInstance<'env, Sample>,
}

#[allow(dead_code)]
#[napi(object)]
pub struct ReplyErrored<'env> {
  #[napi(ts_type = "false")]
  pub is_sample: bool,
  #[napi(ts_type = "true")]
  pub is_error: bool,
  pub error: ClassInstance<'env, ReplyError>,
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

option_wrapper!(
  zquery::Queryable<HandlerImpl<zquery::Query>>,
  "Undeclared queryable"
);

#[napi]
impl Queryable {
  #[napi(getter)]
  pub fn id(&self) -> napi::Result<EntityGlobalId> {
    Ok(self.get_ref()?.id().into())
  }

  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.get_ref()?.key_expr().clone().into())
  }

  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    Wait::wait(self.take()?.undeclare()).map_napi_err()
  }
}

recv_handler!(Queryable => Query);
async_stream!(Queryable => QueryStream yields Query from zquery::Query);

option_wrapper!(zquery::Querier<'static>, "Undeclared querier");

#[napi]
impl Querier {
  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.get_ref()?.key_expr().clone().into())
  }

  #[napi(getter)]
  pub fn id(&self) -> napi::Result<EntityGlobalId> {
    Ok(self.get_ref()?.id().into())
  }

  #[napi(getter)]
  pub fn congestion_control(&self) -> napi::Result<CongestionControl> {
    Ok(self.get_ref()?.congestion_control().into())
  }

  #[napi(getter)]
  pub fn priority(&self) -> napi::Result<Priority> {
    Ok(self.get_ref()?.priority().into())
  }

  #[napi(getter)]
  pub fn accept_replies(&self) -> napi::Result<ReplyKeyExpr> {
    Ok(self.get_ref()?.accept_replies().into())
  }

  #[napi]
  pub async fn get(&self, options: Option<QuerierGetOptions>) -> napi::Result<ReplyHandler> {
    let QuerierGetOptions {
      parameters,
      payload,
      encoding,
      attachment,
      source_info,
      cancellation_token,
      channel,
    } = options.unwrap_or_default();

    let handler = into_handler(channel);

    let receiver = build!(
      self.get_ref()?.get().with(handler),
      parameters,
      payload,
      encoding,
      attachment,
      source_info,
      cancellation_token,
    )
    .await
    .map_napi_err()?;

    Ok(receiver.into())
  }

  #[napi]
  pub async fn matching_status(&self) -> napi::Result<MatchingStatus> {
    let status = self.get_ref()?.matching_status().await.map_napi_err()?;

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
      .get_ref()?
      .matching_listener()
      .with(handler)
      .await
      .map_napi_err()?;

    Ok(listener.into())
  }

  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    Wait::wait(self.take()?.undeclare()).map_napi_err()
  }
}

wrapper!(zquery::Selector<'static>);

impl Selector {
  pub(crate) fn resolve(
    selector: SelectorArg<'_>,
    parameters: Option<Instance<Parameters>>,
  ) -> napi::Result<zquery::Selector<'static>> {
    let mut selector = zquery::Selector::from(Selector::try_from(selector)?);
    if let Some(parameters) = parameters {
      let key_expr = selector.key_expr().clone().into_owned();
      selector = zquery::Selector::owned(key_expr, parameters.into_zenoh());
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

    let parameters = parameters.map(zquery::Parameters::from).unwrap_or_default();

    let inner = zquery::Selector::owned(key_expr, parameters);

    Ok(inner.into())
  }

  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    self.inner.key_expr().clone().into_owned().into()
  }

  #[napi(getter)]
  pub fn parameters(&self) -> Parameters {
    self.inner.parameters().clone().into_owned().into()
  }

  #[napi]
  pub fn split(&self) -> SelectorParts {
    SelectorParts {
      key_expr: self.inner.key_expr().as_str().to_string(),
      parameters: self.inner.parameters().as_str().to_string(),
    }
  }
}

#[napi]
pub type SelectorArg<'a> = Either3<String, &'a KeyExpr, &'a Selector>;

impl TryFrom<SelectorArg<'_>> for Selector {
  type Error = napi::Error;

  fn try_from(value: SelectorArg<'_>) -> napi::Result<Self> {
    match value {
      Either3::A(selector) => zquery::Selector::try_from(selector)
        .map(Selector::from)
        .map_napi_err(),
      Either3::B(key_expr) => {
        Ok(zquery::Selector::from(zenoh::key_expr::KeyExpr::from(key_expr.clone())).into())
      }
      Either3::C(selector) => Ok(zquery::Selector::from(selector).into()),
    }
  }
}
