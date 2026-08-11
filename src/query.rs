use std::collections::HashMap;

use derive_more::{AsRef, From, Into};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh as z;

use crate::{
  bytes::{Bytes, BytesBuffer, Encoding},
  config::EntityGlobalId,
  key_expr::{KeyExpr, KeyExprArg},
  matching::{MatchingListener, MatchingStatus},
  options::{
    MatchingListenerOptions, QuerierGetOptions, ReplyDelOptions, ReplyErrOptions, ReplyOptions,
  },
  qos::{CongestionControl, Priority},
  sample::Sample,
  utils::{Declared, MapNapiErr, fifo},
};

/// A query received by a {@link Queryable}.
///
/// It carries everything the querier sent: the selector, and the payload and attachment,
/// if any. Answer it with {@link Query.reply}, {@link Query.replyDel} or
/// {@link Query.replyErr}.
///
/// {@link Query.keyExpr} is not necessarily the key expression to reply on, as it may
/// contain wildcards. A queryable serving `foo/*` may receive a query for `foo/bar` and
/// another for `foo/baz`, and should reply respectively on `foo/bar` and `foo/baz`.
///
/// {@link Query.drop} consumes the query; every other member throws afterwards.
#[napi]
#[derive(From)]
#[from(forward)]
pub struct Query(Declared<z::query::Query>);

#[napi]
impl Query {
  /// The full selector of this query, i.e. its key expression together with its
  /// parameters.
  #[napi(getter)]
  pub fn selector(&self) -> napi::Result<Selector> {
    Ok(self.0.get()?.selector().into_owned().into())
  }

  /// The key expression this query targets.
  ///
  /// It may contain wildcards, so it is not necessarily the key expression to reply on.
  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.0.get()?.key_expr().clone().into())
  }

  /// The payload sent along with this query, or `null` if it carries none.
  #[napi(getter)]
  pub fn payload(&self) -> napi::Result<Option<Bytes>> {
    Ok(self.0.get()?.payload().cloned().map(Into::into))
  }

  /// The encoding of {@link Query.payload}, or `null` if this query carries no payload.
  #[napi(getter)]
  pub fn encoding(&self) -> napi::Result<Option<Encoding>> {
    Ok(self.0.get()?.encoding().cloned().map(Into::into))
  }

  /// The arbitrary user-defined data sent alongside this query, or `null` if there is
  /// none.
  #[napi(getter)]
  pub fn attachment(&self) -> napi::Result<Option<Bytes>> {
    Ok(self.0.get()?.attachment().cloned().map(Into::into))
  }

  /// The priority applied when routing this query.
  #[napi(getter)]
  pub fn priority(&self) -> napi::Result<Priority> {
    Ok(self.0.get()?.priority().into())
  }

  /// The congestion control applied when routing this query.
  #[napi(getter)]
  pub fn congestion_control(&self) -> napi::Result<CongestionControl> {
    Ok(self.0.get()?.congestion_control().into())
  }

  /// Whether this query was sent without batching, which usually reduces latency.
  #[napi(getter)]
  pub fn express(&self) -> napi::Result<bool> {
    Ok(self.0.get()?.express())
  }

  /// The selector parameters of this query.
  #[napi(getter)]
  pub fn parameters(&self) -> napi::Result<Parameters<'_>> {
    Ok(self.0.get()?.parameters().clone().into())
  }

  /// Whether this query accepts replies whose key expression does not intersect its own.
  ///
  /// See {@link ReplyKeyExpr}.
  #[napi(getter)]
  pub fn accepts_replies(&self) -> napi::Result<ReplyKeyExpr> {
    Ok(self.0.get()?.accepts_replies().into())
  }

  /// Replies to this query with a sample of kind `Put`.
  ///
  /// `keyExpr` is the concrete key expression the replied data belongs to. It is not
  /// necessarily {@link Query.keyExpr}, which may contain wildcards.
  ///
  /// The reply is sent with the `QoS` of the query.
  ///
  /// @throws If `keyExpr` does not intersect the query's key expression while the query
  /// only accepts matching replies, or if the query has already been consumed.
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
      builder = builder.encoding(encoding);
    }

    if let Some(express) = express {
      builder = builder.express(express);
    }

    if let Some(attachment) = attachment {
      builder = builder.attachment(attachment);
    }

    builder.await.map_napi_err()
  }

  /// Replies to this query with an error.
  ///
  /// The reply is sent with the `QoS` of the query.
  ///
  /// @throws If the query has already been consumed.
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

  /// Replies to this query with a sample of kind `Delete`.
  ///
  /// `keyExpr` is the concrete key expression the deletion applies to. It is not
  /// necessarily {@link Query.keyExpr}, which may contain wildcards.
  ///
  /// The reply is sent with the `QoS` of the query.
  ///
  /// @throws If `keyExpr` does not intersect the query's key expression while the query
  /// only accepts matching replies, or if the query has already been consumed.
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
      builder = builder.express(express);
    }

    if let Some(attachment) = attachment {
      builder = builder.attachment(attachment);
    }

    builder.await.map_napi_err()
  }

  /// Releases this query without replying to it.
  ///
  /// @throws If the query has already been consumed.
  #[napi]
  pub fn drop(&mut self) -> napi::Result<()> {
    self.0.take()?;
    Ok(())
  }
}

/// The kinds of replies a query accepts.
///
/// A queryable may serve a glob-like key expression such as `foo/*` while replying on
/// more specific ones. It may therefore receive a query for `foo/bar` and reply on
/// `foo/baz`. By default such disjoint replies are rejected on the sending side.
#[napi(string_enum)]
pub enum ReplyKeyExpr {
  /// Accepts replies whose key expression may not match the query's key expression.
  Any,
  /// Accepts only replies whose key expression matches the query's key expression.
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

/// An answer received from a {@link Queryable}.
///
/// A reply holds either a successful {@link Sample} or a {@link ReplyError}; read
/// {@link Reply.result} to tell them apart.
#[napi]
#[derive(From)]
pub struct Reply(z::query::Reply);

#[napi]
impl Reply {
  /// The global id of the Zenoh entity that answered this reply, or `null` if it is not
  /// known.
  #[napi(getter)]
  pub fn id(&self) -> Option<EntityGlobalId> {
    self.0.replier_id().map(EntityGlobalId::from)
  }

  /// The result carried by this reply.
  ///
  /// Exactly one of its two fields is set: on success `sample` holds the replied data and
  /// `error` is `null`, on failure `error` holds the error and `sample` is `null`.
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

/// The successful variant of {@link ReplyResult}.
#[napi(object)]
pub struct ReplyResultSample<'env> {
  /// The data replied by the queryable.
  pub sample: ClassInstance<'env, Sample>,
  /// Always `null` on this variant.
  pub error: Null,
}

/// The failed variant of {@link ReplyResult}.
#[napi(object)]
pub struct ReplyResultError<'env> {
  /// Always `null` on this variant.
  pub sample: Null,
  /// The error replied by the queryable.
  pub error: ClassInstance<'env, ReplyError>,
}

/// The result carried by a {@link Reply}, either data or an error.
#[napi]
pub type ReplyResult<'env> = Either<ReplyResultSample<'env>, ReplyResultError<'env>>;

/// The error variant of a {@link Reply}.
///
/// It carries the payload describing the error, which may be a message or structured
/// data, along with the encoding of that payload.
#[napi]
#[derive(From)]
pub struct ReplyError(z::query::ReplyError);

#[napi]
impl ReplyError {
  /// The encoding of {@link ReplyError.payload}.
  #[napi(getter)]
  pub fn encoding(&self) -> Encoding {
    self.0.encoding().clone().into()
  }

  /// The payload describing the error.
  #[napi(getter)]
  pub fn payload(&self) -> Bytes {
    self.0.payload().clone().into()
  }
}

/// Selector parameters given as a record of key-value pairs.
#[napi]
pub type ParametersLike = HashMap<String, String>;

impl From<ParametersLike> for Parameters<'_> {
  fn from(value: ParametersLike) -> Self {
    Parameters::from(value)
  }
}

/// The parameters part of a {@link Selector}.
///
/// Parameters are a `;`-separated list of entries, where `=` separates a key from its
/// value and `|` separates the several values of one key. An entry without `=` has the
/// empty string as its value.
///
/// Zenoh reserves the parameter names starting with a non-alphanumeric character, such as
/// `_time` and `_anyke`. Queryables are encouraged to prefix their own parameter names to
/// avoid conflicting with other queryables.
#[napi]
#[derive(Clone, From, Into)]
pub struct Parameters<'s>(z::query::Parameters<'s>);

#[napi]
impl Parameters<'_> {
  /// Creates empty parameters.
  #[napi(factory)]
  pub fn empty() -> Self {
    z::query::Parameters::empty().into()
  }

  /// Creates parameters from their string form, e.g. `a=1;b=2|3;c`.
  #[napi(constructor)]
  pub fn new(params: String) -> Self {
    z::query::Parameters::from(params).into()
  }

  /// Creates parameters from a record of key-value pairs.
  #[napi(factory)]
  pub fn from(params: ParametersLike) -> Self {
    z::query::Parameters::from(params).into()
  }

  /// Returns these parameters in their string form.
  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.0.as_str().to_string()
  }

  /// Whether these parameters hold no entry at all.
  #[napi(getter)]
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  /// Whether the keys of these parameters are sorted in alphabetical order.
  #[napi(getter)]
  pub fn is_ordered(&self) -> bool {
    self.0.is_ordered()
  }

  /// Returns `true` if these parameters contain the given key.
  #[napi]
  pub fn contains_key(&self, key: String) -> bool {
    self.0.contains_key(key)
  }

  /// Returns the value of the given key, or `null` if the key is absent.
  ///
  /// The value is returned as stored, so a key with several `|`-separated values yields
  /// all of them as a single string. Use {@link Parameters.values} to get them apart.
  #[napi]
  pub fn get(&self, key: String) -> Option<String> {
    self.0.get(key).map(std::string::ToString::to_string)
  }

  /// Returns the `|`-separated values of the given key, or an empty array if the key is
  /// absent.
  #[napi]
  pub fn values(&self, key: String) -> Vec<String> {
    self
      .0
      .values(key)
      .map(std::string::ToString::to_string)
      .collect()
  }

  /// Inserts a key-value pair into these parameters.
  ///
  /// @returns The value previously held by that key, or `null` if the key was absent.
  #[napi]
  pub fn insert(&mut self, key: String, value: String) -> Option<String> {
    self.0.insert(key, value)
  }

  /// Removes a key from these parameters.
  ///
  /// @returns The value that key held, or `null` if the key was absent.
  #[napi]
  pub fn remove(&mut self, key: String) -> Option<String> {
    self.0.remove(key)
  }

  /// Extends these parameters with the entries of `other`.
  ///
  /// Keys held by both sides take the value from `other`.
  #[napi]
  pub fn extend(&mut self, other: &Parameters) {
    self.0.extend(&other.0);
  }
}

/// The queryables a query is delivered to.
///
/// See also {@link QueryableOptions.complete}.
#[napi(string_enum)]
pub enum QueryTarget {
  /// Requests the data from the queryable(s) Zenoh selects to get the fastest and most
  /// complete reply.
  BestMatching,
  /// Delivers the query to all the matching queryables.
  All,
  /// Delivers the query to all the matching queryables that are declared as complete.
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

/// The strategy applied to filter and reorder the replies to a query.
///
/// Several replies may arrive for the same key, from the same or from different
/// queryables.
#[napi(string_enum)]
pub enum ConsolidationMode {
  /// Applies the consolidation Zenoh deems best given the query and the responders.
  Auto,
  /// Applies no consolidation: several replies may be received for the same key and
  /// timestamp.
  None,
  /// Forwards replies immediately, except those for a key on which a reply with an equal
  /// or more recent timestamp was already forwarded.
  ///
  /// This optimizes latency while potentially reducing bandwidth. It does not reorder
  /// replies.
  Monotonic,
  /// Holds replies back to only deliver, for each key, the one with the highest
  /// timestamp.
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

/// An entity that implements the query/reply pattern.
///
/// A queryable receives the queries sent by {@link Session.get} and {@link Querier.get}
/// that match its key expression, and answers them through the methods of {@link Query}.
///
/// {@link Queryable.undeclare} consumes the queryable; every other member throws
/// afterwards.
#[napi]
#[derive(From)]
#[from(forward)]
pub struct Queryable(
  Declared<z::query::Queryable<z::handlers::FifoChannelHandler<z::query::Query>>>,
);

#[napi]
impl Queryable {
  /// The global id of this queryable.
  #[napi(getter)]
  pub fn id(&self) -> napi::Result<EntityGlobalId> {
    Ok(self.0.get()?.id().into())
  }

  /// The key expression this queryable answers queries for.
  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.0.get()?.key_expr().clone().into())
  }

  /// Undeclares this queryable, stopping the delivery of queries to it.
  ///
  /// @throws If the queryable has already been undeclared.
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    z::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }

  /// Returns the stream of the queries delivered to this queryable.
  ///
  /// Iteration ends once the queryable is undeclared.
  ///
  /// @throws If the queryable has already been undeclared.
  #[napi]
  pub fn receive(&self) -> napi::Result<QueryIter> {
    let handler = self.0.get()?.handler().clone();

    Ok(handler.into())
  }
}

/// A stream of the queries delivered to a queryable.
#[napi(async_iterator)]
#[derive(From)]
pub struct QueryIter(z::handlers::FifoChannelHandler<z::query::Query>);

#[napi]
impl AsyncGenerator for QueryIter {
  type Yield = Query;
  type Next = ();
  type Return = ();

  fn next(
    &mut self,
    _value: Option<Self::Next>,
  ) -> impl Future<Output = napi::Result<Option<Self::Yield>>> + Send + 'static {
    let receiver = self.0.clone();

    async move {
      match receiver.recv_async().await {
        Ok(query) => Ok(Some(query.into())),
        Err(_) => Ok(None),
      }
    }
  }
}

/// A stream of the replies to a single query.
#[napi]
#[derive(From)]
pub struct Replies(z::handlers::FifoChannelHandler<z::query::Reply>);

#[napi]
impl Replies {
  /// Returns the stream of the replies to this query.
  ///
  /// Iteration ends once the query completes, i.e. once every matching queryable has
  /// answered or the query has timed out.
  #[napi]
  pub fn receive(&self) -> ReplyIter {
    self.0.clone().into()
  }
}

/// A stream of the replies to a single query.
#[napi(async_iterator)]
#[derive(From)]
pub struct ReplyIter(z::handlers::FifoChannelHandler<z::query::Reply>);

#[napi]
impl AsyncGenerator for ReplyIter {
  type Yield = Reply;
  type Next = ();
  type Return = ();

  fn next(
    &mut self,
    _value: Option<Self::Next>,
  ) -> impl Future<Output = napi::Result<Option<Self::Yield>>> + Send + 'static {
    let receiver = self.0.clone();

    async move {
      match receiver.recv_async().await {
        Ok(reply) => Ok(Some(reply.into())),
        Err(_) => Ok(None),
      }
    }
  }
}

/// A preconfigured sender of queries to a given key expression.
///
/// Declaring a querier lets Zenoh optimize the routing of the queries it sends, which is
/// worthwhile when the same key expression is queried repeatedly.
///
/// {@link Querier.undeclare} consumes the querier; every other member throws afterwards.
#[napi]
#[derive(From)]
#[from(forward)]
pub struct Querier(Declared<z::query::Querier<'static>>);

#[napi]
impl Querier {
  /// The key expression this querier sends its queries on.
  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.0.get()?.key_expr().clone().into())
  }

  /// The global id of this querier.
  #[napi(getter)]
  pub fn id(&self) -> napi::Result<EntityGlobalId> {
    Ok(self.0.get()?.id().into())
  }

  /// The congestion control applied when routing the queries.
  #[napi(getter)]
  pub fn congestion_control(&self) -> napi::Result<CongestionControl> {
    Ok(self.0.get()?.congestion_control().into())
  }

  /// The priority applied when routing the queries.
  #[napi(getter)]
  pub fn priority(&self) -> napi::Result<Priority> {
    Ok(self.0.get()?.priority().into())
  }

  /// Whether this querier accepts replies whose key expression does not intersect its
  /// own.
  ///
  /// See {@link ReplyKeyExpr}.
  #[napi(getter)]
  pub fn accept_replies(&self) -> napi::Result<ReplyKeyExpr> {
    Ok(self.0.get()?.accept_replies().into())
  }

  /// Sends a query on this querier's key expression.
  ///
  /// @returns The stream of the replies received for this query.
  ///
  /// @throws If the querier has already been undeclared.
  #[napi]
  pub async fn get(&self, options: Option<QuerierGetOptions>) -> napi::Result<Replies> {
    let QuerierGetOptions {
      parameters,
      payload,
      encoding,
      attachment,
      // cancellation_token,
      channel_capacity,
    } = options.unwrap_or_default();
    let handler = fifo(channel_capacity);
    let mut builder = self.0.get()?.get().with(handler);

    if let Some(parameters) = parameters {
      builder = builder.parameters(Parameters::from(parameters));
    }

    if let Some(payload) = payload {
      builder = builder.payload(payload);
    }

    if let Some(encoding) = encoding {
      builder = builder.encoding(encoding);
    }

    if let Some(attachment) = attachment {
      builder = builder.attachment(attachment);
    }

    let receiver = builder.await.map_napi_err()?;

    Ok(receiver.into())
  }

  /// Returns whether there is currently at least one queryable matching this querier's
  /// key expression and target.
  ///
  /// @throws If the querier has already been undeclared.
  #[napi]
  pub async fn matching_status(&self) -> napi::Result<MatchingStatus> {
    let status = self.0.get()?.matching_status().await.map_napi_err()?;

    Ok(status.into())
  }

  /// Declares a listener notified each time the matching status of this querier changes.
  ///
  /// @throws If the querier has already been undeclared.
  #[napi]
  pub async fn matching_listener(
    &self,
    options: Option<MatchingListenerOptions>,
  ) -> napi::Result<MatchingListener> {
    let MatchingListenerOptions { channel_capacity } = options.unwrap_or_default();
    let handler = fifo(channel_capacity);

    let listener = self
      .0
      .get()?
      .matching_listener()
      .with(handler)
      .await
      .map_napi_err()?;

    Ok(listener.into())
  }

  /// Undeclares this querier, informing the network that it no longer needs to optimize
  /// queries for its key expression.
  ///
  /// @throws If the querier has already been undeclared.
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    z::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }
}

/// The combination of a key expression and a set of parameters, identifying what a query
/// targets.
///
/// The key expression defines the set of keys the query is relevant to, and the
/// parameters carry arguments to the queryables answering it, e.g. remote procedure call
/// arguments or filters on value or metadata.
///
/// In string form a selector looks like a URI: the part before the first `?` is the key
/// expression and the part after it is the parameters.
#[napi]
#[derive(AsRef, From, Into)]
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

/// The two parts of a {@link Selector}, in string form.
#[napi(object)]
pub struct SelectorParts {
  /// The key expression, i.e. everything before the first `?`.
  pub key_expr: String,
  /// The parameters, i.e. everything after the first `?`.
  pub parameters: String,
}

#[napi]
impl Selector {
  /// Creates a selector from a key expression and, optionally, parameters in their string
  /// form.
  ///
  /// @throws If `keyExpr` is a string that is not a valid key expression.
  #[napi(constructor)]
  pub fn new(key_expr: KeyExprArg, parameters: Option<String>) -> napi::Result<Self> {
    let key_expr = KeyExpr::try_from(key_expr)?;

    let parameters = parameters
      .map(z::query::Parameters::from)
      .unwrap_or_default();

    let inner = z::query::Selector::owned(key_expr, parameters);

    Ok(inner.into())
  }

  /// The key expression of this selector, defining the set of keys the query is relevant
  /// to.
  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    self.0.key_expr().clone().into_owned().into()
  }

  /// The parameters of this selector.
  #[napi(getter)]
  pub fn parameters(&self) -> Parameters<'_> {
    self.0.parameters().clone().into_owned().into()
  }

  /// Returns the key expression and the parameters of this selector, both in their string
  /// form.
  #[napi]
  pub fn split(&self) -> SelectorParts {
    SelectorParts {
      key_expr: self.0.key_expr().as_str().to_string(),
      parameters: self.0.parameters().as_str().to_string(),
    }
  }
}

/// A {@link Selector}, a {@link KeyExpr} to use as one, or the string form of either.
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
