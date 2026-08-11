use derive_more::From;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh as z;

/// A payload accepted wherever raw bytes are sent.
///
/// A string is carried as its UTF-8 bytes; a `Uint8Array` is carried as-is.
#[napi(transparent)]
#[derive(From)]
pub struct BytesBuffer(napi::Either<String, Uint8Array>);

impl From<BytesBuffer> for z::bytes::ZBytes {
  fn from(value: BytesBuffer) -> Self {
    match value.0 {
      napi::Either::A(s) => z::bytes::ZBytes::from(s),
      napi::Either::B(bytes) => z::bytes::ZBytes::from(bytes.to_vec()),
    }
  }
}

/// The raw bytes carried by a sample, a reply or an attachment.
///
/// Zenoh does not interpret the payload; how it should be read is conveyed separately by
/// an {@link Encoding}.
#[napi]
#[derive(From)]
pub struct Bytes(z::bytes::ZBytes);

#[napi]
impl Bytes {
  /// Creates an empty payload.
  #[napi(constructor)]
  pub fn new() -> Self {
    z::bytes::ZBytes::new().into()
  }

  /// Creates a payload from a string or a byte array.
  #[napi(factory)]
  pub fn from(value: BytesBuffer) -> Self {
    let zbytes = match value.0 {
      napi::Either::A(s) => z::bytes::ZBytes::from(s),
      napi::Either::B(bytes) => z::bytes::ZBytes::from(bytes.to_vec()),
    };
    zbytes.into()
  }

  /// Whether this payload carries no bytes.
  #[napi(getter)]
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  /// The total number of bytes in this payload.
  #[napi(getter)]
  #[allow(clippy::cast_possible_truncation)]
  pub fn len(&self) -> u32 {
    self.0.len() as u32
  }

  /// Copies the payload into a byte array.
  ///
  /// Zenoh may hold a payload received from the network across several memory regions,
  /// so the bytes are gathered into a newly allocated `Uint8Array`.
  #[napi]
  pub fn to_bytes(&self) -> Uint8Array {
    Uint8Array::from(self.0.to_bytes().into_owned())
  }

  /// Decodes the payload as a UTF-8 string.
  ///
  /// Non-UTF-8 sequences are replaced with U+FFFD (`�`), one per maximal
  /// invalid subsequence. Use {@link Bytes.toBytes} for arbitrary bytes.
  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    String::from_utf8_lossy(&self.0.to_bytes()).into_owned()
  }
}

/// How the payload of a sample or a reply should be interpreted by the application.
///
/// An encoding is a string in MIME-like format, `type/subtype[;schema]`. Zenoh neither
/// imposes an encoding value nor operates on it: it is optional metadata carried
/// alongside the payload so that the receiving application can decide what to do with it.
///
/// Some encodings are mapped internally to a compact integer identifier and are therefore
/// cheaper to send than arbitrary strings; those are the ones exposed as the static
/// factories below.
#[napi]
#[derive(From)]
pub struct Encoding(z::bytes::Encoding);

#[napi]
impl Encoding {
  /// The default encoding, `zenoh/bytes`.
  #[napi(factory)]
  pub fn default() -> Self {
    z::bytes::Encoding::default().into()
  }

  /// Creates an encoding from its string representation, `type/subtype[;schema]`.
  ///
  /// A value that is not one of the encodings Zenoh knows about is carried as-is.
  #[napi(factory)]
  pub fn from(value: String) -> Self {
    z::bytes::Encoding::from(value).into()
  }

  /// Just some bytes, with no assumption made about their format: `zenoh/bytes`.
  #[napi(factory)]
  pub fn zenoh_bytes() -> Self {
    z::bytes::Encoding::ZENOH_BYTES.into()
  }

  /// A UTF-8 string: `zenoh/string`.
  #[napi(factory)]
  pub fn zenoh_string() -> Self {
    z::bytes::Encoding::ZENOH_STRING.into()
  }

  /// Data serialized by a Zenoh binding: `zenoh/serialized`.
  #[napi(factory)]
  pub fn zenoh_serialized() -> Self {
    z::bytes::Encoding::ZENOH_SERIALIZED.into()
  }

  /// An application-specific stream of bytes: `application/octet-stream`.
  #[napi(factory)]
  pub fn application_octet_stream() -> Self {
    z::bytes::Encoding::APPLICATION_OCTET_STREAM.into()
  }

  /// A textual file: `text/plain`.
  #[napi(factory)]
  pub fn text_plain() -> Self {
    z::bytes::Encoding::TEXT_PLAIN.into()
  }

  /// JSON data intended to be consumed by an application: `application/json`.
  #[napi(factory)]
  pub fn application_json() -> Self {
    z::bytes::Encoding::APPLICATION_JSON.into()
  }

  /// JSON data intended to be human readable: `text/json`.
  #[napi(factory)]
  pub fn text_json() -> Self {
    z::bytes::Encoding::TEXT_JSON.into()
  }

  /// Common Data Representation (CDR)-encoded data: `application/cdr`.
  #[napi(factory)]
  pub fn application_cdr() -> Self {
    z::bytes::Encoding::APPLICATION_CDR.into()
  }

  /// Concise Binary Object Representation (CBOR)-encoded data: `application/cbor`.
  #[napi(factory)]
  pub fn application_cbor() -> Self {
    z::bytes::Encoding::APPLICATION_CBOR.into()
  }

  /// YAML data intended to be consumed by an application: `application/yaml`.
  #[napi(factory)]
  pub fn application_yaml() -> Self {
    z::bytes::Encoding::APPLICATION_YAML.into()
  }

  /// YAML data intended to be human readable: `text/yaml`.
  #[napi(factory)]
  pub fn text_yaml() -> Self {
    z::bytes::Encoding::TEXT_YAML.into()
  }

  /// JSON5-encoded data intended to be human readable: `text/json5`.
  #[napi(factory)]
  pub fn text_json5() -> Self {
    z::bytes::Encoding::TEXT_JSON5.into()
  }

  /// A Python object serialized with `pickle`: `application/python-serialized-object`.
  #[napi(factory)]
  pub fn application_python_serialized_object() -> Self {
    z::bytes::Encoding::APPLICATION_PYTHON_SERIALIZED_OBJECT.into()
  }

  /// Application-specific protobuf-encoded data: `application/protobuf`.
  #[napi(factory)]
  pub fn application_protobuf() -> Self {
    z::bytes::Encoding::APPLICATION_PROTOBUF.into()
  }

  /// A Java serialized object: `application/java-serialized-object`.
  #[napi(factory)]
  pub fn application_java_serialized_object() -> Self {
    z::bytes::Encoding::APPLICATION_JAVA_SERIALIZED_OBJECT.into()
  }

  /// `OpenMetrics` data, commonly used by Prometheus: `application/openmetrics-text`.
  #[napi(factory)]
  pub fn application_openmetrics_text() -> Self {
    z::bytes::Encoding::APPLICATION_OPENMETRICS_TEXT.into()
  }

  /// A Portable Network Graphics (PNG) image: `image/png`.
  #[napi(factory)]
  pub fn image_png() -> Self {
    z::bytes::Encoding::IMAGE_PNG.into()
  }

  /// A Joint Photographic Experts Group (JPEG) image: `image/jpeg`.
  #[napi(factory)]
  pub fn image_jpeg() -> Self {
    z::bytes::Encoding::IMAGE_JPEG.into()
  }

  /// A Graphics Interchange Format (GIF) image: `image/gif`.
  #[napi(factory)]
  pub fn image_gif() -> Self {
    z::bytes::Encoding::IMAGE_GIF.into()
  }

  /// A Bitmap (BMP) image: `image/bmp`.
  #[napi(factory)]
  pub fn image_bmp() -> Self {
    z::bytes::Encoding::IMAGE_BMP.into()
  }

  /// A WebP image: `image/webp`.
  #[napi(factory)]
  pub fn image_webp() -> Self {
    z::bytes::Encoding::IMAGE_WEBP.into()
  }

  /// An XML file intended to be consumed by an application: `application/xml`.
  #[napi(factory)]
  pub fn application_xml() -> Self {
    z::bytes::Encoding::APPLICATION_XML.into()
  }

  /// An encoded list of name/value tuples: `application/x-www-form-urlencoded`.
  #[napi(factory)]
  pub fn application_x_www_form_urlencoded() -> Self {
    z::bytes::Encoding::APPLICATION_X_WWW_FORM_URLENCODED.into()
  }

  /// An HTML file: `text/html`.
  #[napi(factory)]
  pub fn text_html() -> Self {
    z::bytes::Encoding::TEXT_HTML.into()
  }

  /// An XML file that is human readable: `text/xml`.
  #[napi(factory)]
  pub fn text_xml() -> Self {
    z::bytes::Encoding::TEXT_XML.into()
  }

  /// A CSS file: `text/css`.
  #[napi(factory)]
  pub fn text_css() -> Self {
    z::bytes::Encoding::TEXT_CSS.into()
  }

  /// A JavaScript file: `text/javascript`.
  #[napi(factory)]
  pub fn text_javascript() -> Self {
    z::bytes::Encoding::TEXT_JAVASCRIPT.into()
  }

  /// A Markdown file: `text/markdown`.
  #[napi(factory)]
  pub fn text_markdown() -> Self {
    z::bytes::Encoding::TEXT_MARKDOWN.into()
  }

  /// A CSV file: `text/csv`.
  #[napi(factory)]
  pub fn text_csv() -> Self {
    z::bytes::Encoding::TEXT_CSV.into()
  }

  /// An application-specific SQL query: `application/sql`.
  #[napi(factory)]
  pub fn application_sql() -> Self {
    z::bytes::Encoding::APPLICATION_SQL.into()
  }

  /// Constrained Application Protocol (CoAP) data intended for CoAP-to-HTTP and
  /// HTTP-to-CoAP proxies: `application/coap-payload`.
  #[napi(factory)]
  pub fn application_coap_payload() -> Self {
    z::bytes::Encoding::APPLICATION_COAP_PAYLOAD.into()
  }

  /// A sequence of operations to apply to a JSON document: `application/json-patch+json`.
  #[napi(factory)]
  pub fn application_json_patch_json() -> Self {
    z::bytes::Encoding::APPLICATION_JSON_PATCH_JSON.into()
  }

  /// A sequence of UTF-8 encoded JSON texts: `application/json-seq`.
  #[napi(factory)]
  pub fn application_json_seq() -> Self {
    z::bytes::Encoding::APPLICATION_JSON_SEQ.into()
  }

  /// A `JSONPath` expression selecting values within a JSON value: `application/jsonpath`.
  #[napi(factory)]
  pub fn application_jsonpath() -> Self {
    z::bytes::Encoding::APPLICATION_JSONPATH.into()
  }

  /// A JSON Web Token (JWT): `application/jwt`.
  #[napi(factory)]
  pub fn application_jwt() -> Self {
    z::bytes::Encoding::APPLICATION_JWT.into()
  }

  /// Application-specific MPEG-4-encoded data, either audio or video: `application/mp4`.
  #[napi(factory)]
  pub fn application_mp4() -> Self {
    z::bytes::Encoding::APPLICATION_MP4.into()
  }

  /// A SOAP 1.2 message serialized as XML 1.0: `application/soap+xml`.
  #[napi(factory)]
  pub fn application_soap_xml() -> Self {
    z::bytes::Encoding::APPLICATION_SOAP_XML.into()
  }

  /// YANG-encoded data, commonly used by the Network Configuration Protocol (NETCONF):
  /// `application/yang`.
  #[napi(factory)]
  pub fn application_yang() -> Self {
    z::bytes::Encoding::APPLICATION_YANG.into()
  }

  /// An MPEG-4 Advanced Audio Coding (AAC) media: `audio/aac`.
  #[napi(factory)]
  pub fn audio_aac() -> Self {
    z::bytes::Encoding::AUDIO_AAC.into()
  }

  /// A Free Lossless Audio Codec (FLAC) media: `audio/flac`.
  #[napi(factory)]
  pub fn audio_flac() -> Self {
    z::bytes::Encoding::AUDIO_FLAC.into()
  }

  /// An audio codec defined in MPEG-1, MPEG-2 or MPEG-4: `audio/mp4`.
  #[napi(factory)]
  pub fn audio_mp4() -> Self {
    z::bytes::Encoding::AUDIO_MP4.into()
  }

  /// An Ogg-encapsulated audio stream: `audio/ogg`.
  #[napi(factory)]
  pub fn audio_ogg() -> Self {
    z::bytes::Encoding::AUDIO_OGG.into()
  }

  /// A Vorbis-encoded audio stream: `audio/vorbis`.
  #[napi(factory)]
  pub fn audio_vorbis() -> Self {
    z::bytes::Encoding::AUDIO_VORBIS.into()
  }

  /// An h261-encoded video stream: `video/h261`.
  #[napi(factory)]
  pub fn video_h261() -> Self {
    z::bytes::Encoding::VIDEO_H261.into()
  }

  /// An h263-encoded video stream: `video/h263`.
  #[napi(factory)]
  pub fn video_h263() -> Self {
    z::bytes::Encoding::VIDEO_H263.into()
  }

  /// An h264-encoded video stream: `video/h264`.
  #[napi(factory)]
  pub fn video_h264() -> Self {
    z::bytes::Encoding::VIDEO_H264.into()
  }

  /// An h265-encoded video stream: `video/h265`.
  #[napi(factory)]
  pub fn video_h265() -> Self {
    z::bytes::Encoding::VIDEO_H265.into()
  }

  /// An h266-encoded video stream: `video/h266`.
  #[napi(factory)]
  pub fn video_h266() -> Self {
    z::bytes::Encoding::VIDEO_H266.into()
  }

  /// A video codec defined in MPEG-1, MPEG-2 or MPEG-4: `video/mp4`.
  #[napi(factory)]
  pub fn video_mp4() -> Self {
    z::bytes::Encoding::VIDEO_MP4.into()
  }

  /// An Ogg-encapsulated video stream: `video/ogg`.
  #[napi(factory)]
  pub fn video_ogg() -> Self {
    z::bytes::Encoding::VIDEO_OGG.into()
  }

  /// An uncompressed, studio-quality video stream: `video/raw`.
  #[napi(factory)]
  pub fn video_raw() -> Self {
    z::bytes::Encoding::VIDEO_RAW.into()
  }

  /// A VP8-encoded video stream: `video/vp8`.
  #[napi(factory)]
  pub fn video_vp8() -> Self {
    z::bytes::Encoding::VIDEO_VP8.into()
  }

  /// A VP9-encoded video stream: `video/vp9`.
  #[napi(factory)]
  pub fn video_vp9() -> Self {
    z::bytes::Encoding::VIDEO_VP9.into()
  }

  /// The string representation of this encoding, e.g. `text/plain;utf-8`.
  // Exposed to JS as `toString()`; napi cannot surface a `Display` impl, so the
  // inherent method is deliberate despite clippy's `inherent_to_string` lint.
  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.0.to_string()
  }

  /// Returns a copy of this encoding carrying the given schema.
  ///
  /// Zenoh does not define what a schema is; its meaning is left to the application. A
  /// common schema for `text/plain`, for instance, is `utf-8`, which renders as
  /// `text/plain;utf-8`.
  #[napi]
  pub fn with_schema(&self, value: String) -> Self {
    z::bytes::Encoding::from(self.0.to_string())
      .with_schema(value)
      .into()
  }
}
