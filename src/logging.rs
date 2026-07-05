#![cfg_attr(test, allow(dead_code))]
use std::{fmt, thread};

use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use tracing::{Event, Metadata, Subscriber, field::Field, span};
use tracing_subscriber::{
  layer::{Context, SubscriberExt},
  registry::LookupSpan,
};

#[napi]
pub type LogCallback = ThreadsafeFunction<LogRecord, (), LogRecord, Status, false>;

#[napi(string_enum)]
pub enum LogLevel {
  Trace,
  Debug,
  Info,
  Warn,
  Error,
}

impl LogLevel {
  pub(crate) fn rank(self) -> u8 {
    match self {
      LogLevel::Trace => 0,
      LogLevel::Debug => 1,
      LogLevel::Info => 2,
      LogLevel::Warn => 3,
      LogLevel::Error => 4,
    }
  }
}

impl From<tracing::Level> for LogLevel {
  fn from(level: tracing::Level) -> Self {
    match level {
      tracing::Level::TRACE => LogLevel::Trace,
      tracing::Level::DEBUG => LogLevel::Debug,
      tracing::Level::INFO => LogLevel::Info,
      tracing::Level::WARN => LogLevel::Warn,
      tracing::Level::ERROR => LogLevel::Error,
    }
  }
}

pub(crate) fn level_rank(level: &tracing::Level) -> u8 {
  LogLevel::from(*level).rank()
}

#[napi(object)]
pub struct LogAttribute {
  pub key: String,
  pub value: String,
}

#[napi(object)]
pub struct LogRecord {
  pub level: LogLevel,
  pub target: String,
  pub message: Option<String>,
  pub file: Option<String>,
  pub line: Option<u32>,
  pub thread_id: String,
  pub thread_name: Option<String>,
  pub attributes: Vec<LogAttribute>,
}

#[derive(Clone)]
struct SpanFields(Vec<(&'static str, String)>);

struct Layer<Enabled, Callback> {
  enabled: Enabled,
  callback: Callback,
}

impl<S, E, C> tracing_subscriber::Layer<S> for Layer<E, C>
where
  S: Subscriber + for<'a> LookupSpan<'a>,
  E: Fn(&Metadata) -> bool + 'static,
  C: Fn(LogRecord) + 'static,
{
  fn enabled(&self, metadata: &Metadata<'_>, _: Context<'_, S>) -> bool {
    (self.enabled)(metadata)
  }

  fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
    let span = ctx.span(id).unwrap();
    let mut extensions = span.extensions_mut();
    let mut fields = vec![];
    attrs.record(&mut |field: &Field, value: &dyn fmt::Debug| {
      fields.push((field.name(), format!("{value:?}")))
    });
    extensions.insert(SpanFields(fields));
  }

  fn on_record(&self, id: &span::Id, values: &span::Record<'_>, ctx: Context<'_, S>) {
    let span = ctx.span(id).unwrap();
    let mut extensions = span.extensions_mut();
    let fields = extensions.get_mut::<SpanFields>().unwrap();
    values.record(&mut |field: &Field, value: &dyn fmt::Debug| {
      fields.0.push((field.name(), format!("{value:?}")))
    });
  }

  fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
    let thread = thread::current();
    let metadata = event.metadata();
    let mut record = LogRecord {
      level: (*metadata.level()).into(),
      target: metadata.target().into(),
      message: None,
      file: metadata.file().map(Into::into),
      line: metadata.line(),
      thread_id: format!("{:?}", thread.id()),
      thread_name: thread.name().map(Into::into),
      attributes: vec![],
    };
    if let Some(scope) = ctx.event_scope(event) {
      for span in scope.from_root() {
        let extensions = span.extensions();
        let fields = extensions.get::<SpanFields>().unwrap();
        record
          .attributes
          .extend(fields.0.iter().map(|(key, value)| LogAttribute {
            key: (*key).into(),
            value: value.clone(),
          }));
      }
    }
    event.record(&mut |field: &Field, value: &dyn fmt::Debug| {
      if field.name() == "message" {
        record.message = Some(format!("{value:?}"));
      } else {
        record.attributes.push(LogAttribute {
          key: field.name().into(),
          value: format!("{value:?}"),
        });
      }
    });
    (self.callback)(record);
  }
}

#[napi]
pub fn init_log(callback: LogCallback, level: LogLevel) -> bool {
  let threshold = level.rank();
  let enabled = move |metadata: &Metadata| level_rank(metadata.level()) >= threshold;
  let callback = move |record: LogRecord| {
    callback.call(record, ThreadsafeFunctionCallMode::NonBlocking);
  };
  let subscriber = tracing_subscriber::registry().with(Layer { enabled, callback });
  tracing::subscriber::set_global_default(subscriber).is_ok()
}
