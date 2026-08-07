use derive_more::{AsRef, From, Into};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh as z;

use crate::utils::*;

#[napi]
pub type BytesLike = napi::Either<String, Uint8Array>;

pub(crate) trait IntoZBytes {
  fn into_zbytes(self) -> z::bytes::ZBytes;
}

impl IntoZBytes for BytesLike {
  fn into_zbytes(self) -> z::bytes::ZBytes {
    match self {
      napi::Either::A(s) => z::bytes::ZBytes::from(s),
      napi::Either::B(bytes) => z::bytes::ZBytes::from(bytes.to_vec()),
    }
  }
}

impl IntoZenoh for BytesLike {
  type Into = zenoh::bytes::ZBytes;
  fn into_zenoh(self) -> zenoh::bytes::ZBytes {
    self.into_zbytes()
  }
}

#[derive(AsRef, From, Into)]
#[napi]
pub struct Bytes(z::bytes::ZBytes);

#[napi]
impl Bytes {
  #[napi(constructor)]
  pub fn new() -> Self {
    z::bytes::ZBytes::new().into()
  }

  #[napi(factory)]
  pub fn from(value: BytesLike) -> Self {
    value.into_zbytes().into()
  }

  #[napi(getter)]
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  #[napi(getter)]
  pub fn len(&self) -> u32 {
    self.0.len() as u32
  }

  #[napi]
  pub fn to_bytes(&self) -> Uint8Array {
    Uint8Array::from(self.0.to_bytes().into_owned())
  }

  /// Decodes the payload as a UTF-8 string.
  ///
  /// @returns The decoded string, or `null` if the payload contains non-UTF-8
  /// bytes. use {@link Bytes.toBytes} for arbitrary bytes.
  #[napi]
  pub fn try_to_string(&self) -> Option<String> {
    self.0.try_to_string().ok().map(|s| s.into_owned())
  }
}

#[derive(From, Into)]
#[napi]
pub struct Encoding(z::bytes::Encoding);

#[napi]
impl Encoding {
  #[napi(factory)]
  pub fn default() -> Self {
    z::bytes::Encoding::default().into()
  }

  #[napi(factory)]
  pub fn from(value: String) -> Self {
    z::bytes::Encoding::from(value).into()
  }

  #[napi(factory)]
  pub fn zenoh_bytes() -> Self {
    z::bytes::Encoding::ZENOH_BYTES.into()
  }

  #[napi(factory)]
  pub fn zenoh_string() -> Self {
    z::bytes::Encoding::ZENOH_STRING.into()
  }

  #[napi(factory)]
  pub fn zenoh_serialized() -> Self {
    z::bytes::Encoding::ZENOH_SERIALIZED.into()
  }

  #[napi(factory)]
  pub fn application_octet_stream() -> Self {
    z::bytes::Encoding::APPLICATION_OCTET_STREAM.into()
  }

  #[napi(factory)]
  pub fn text_plain() -> Self {
    z::bytes::Encoding::TEXT_PLAIN.into()
  }

  #[napi(factory)]
  pub fn application_json() -> Self {
    z::bytes::Encoding::APPLICATION_JSON.into()
  }

  #[napi(factory)]
  pub fn text_json() -> Self {
    z::bytes::Encoding::TEXT_JSON.into()
  }

  #[napi(factory)]
  pub fn application_cdr() -> Self {
    z::bytes::Encoding::APPLICATION_CDR.into()
  }

  #[napi(factory)]
  pub fn application_cbor() -> Self {
    z::bytes::Encoding::APPLICATION_CBOR.into()
  }

  #[napi(factory)]
  pub fn application_yaml() -> Self {
    z::bytes::Encoding::APPLICATION_YAML.into()
  }

  #[napi(factory)]
  pub fn text_yaml() -> Self {
    z::bytes::Encoding::TEXT_YAML.into()
  }

  #[napi(factory)]
  pub fn text_json5() -> Self {
    z::bytes::Encoding::TEXT_JSON5.into()
  }

  #[napi(factory)]
  pub fn application_python_serialized_object() -> Self {
    z::bytes::Encoding::APPLICATION_PYTHON_SERIALIZED_OBJECT.into()
  }

  #[napi(factory)]
  pub fn application_protobuf() -> Self {
    z::bytes::Encoding::APPLICATION_PROTOBUF.into()
  }

  #[napi(factory)]
  pub fn application_java_serialized_object() -> Self {
    z::bytes::Encoding::APPLICATION_JAVA_SERIALIZED_OBJECT.into()
  }

  #[napi(factory)]
  pub fn application_openmetrics_text() -> Self {
    z::bytes::Encoding::APPLICATION_OPENMETRICS_TEXT.into()
  }

  #[napi(factory)]
  pub fn image_png() -> Self {
    z::bytes::Encoding::IMAGE_PNG.into()
  }

  #[napi(factory)]
  pub fn image_jpeg() -> Self {
    z::bytes::Encoding::IMAGE_JPEG.into()
  }

  #[napi(factory)]
  pub fn image_gif() -> Self {
    z::bytes::Encoding::IMAGE_GIF.into()
  }

  #[napi(factory)]
  pub fn image_bmp() -> Self {
    z::bytes::Encoding::IMAGE_BMP.into()
  }

  #[napi(factory)]
  pub fn image_webp() -> Self {
    z::bytes::Encoding::IMAGE_WEBP.into()
  }

  #[napi(factory)]
  pub fn application_xml() -> Self {
    z::bytes::Encoding::APPLICATION_XML.into()
  }

  #[napi(factory)]
  pub fn application_x_www_form_urlencoded() -> Self {
    z::bytes::Encoding::APPLICATION_X_WWW_FORM_URLENCODED.into()
  }

  #[napi(factory)]
  pub fn text_html() -> Self {
    z::bytes::Encoding::TEXT_HTML.into()
  }

  #[napi(factory)]
  pub fn text_xml() -> Self {
    z::bytes::Encoding::TEXT_XML.into()
  }

  #[napi(factory)]
  pub fn text_css() -> Self {
    z::bytes::Encoding::TEXT_CSS.into()
  }

  #[napi(factory)]
  pub fn text_javascript() -> Self {
    z::bytes::Encoding::TEXT_JAVASCRIPT.into()
  }

  #[napi(factory)]
  pub fn text_markdown() -> Self {
    z::bytes::Encoding::TEXT_MARKDOWN.into()
  }

  #[napi(factory)]
  pub fn text_csv() -> Self {
    z::bytes::Encoding::TEXT_CSV.into()
  }

  #[napi(factory)]
  pub fn application_sql() -> Self {
    z::bytes::Encoding::APPLICATION_SQL.into()
  }

  #[napi(factory)]
  pub fn application_coap_payload() -> Self {
    z::bytes::Encoding::APPLICATION_COAP_PAYLOAD.into()
  }

  #[napi(factory)]
  pub fn application_json_patch_json() -> Self {
    z::bytes::Encoding::APPLICATION_JSON_PATCH_JSON.into()
  }

  #[napi(factory)]
  pub fn application_json_seq() -> Self {
    z::bytes::Encoding::APPLICATION_JSON_SEQ.into()
  }

  #[napi(factory)]
  pub fn application_jsonpath() -> Self {
    z::bytes::Encoding::APPLICATION_JSONPATH.into()
  }

  #[napi(factory)]
  pub fn application_jwt() -> Self {
    z::bytes::Encoding::APPLICATION_JWT.into()
  }

  #[napi(factory)]
  pub fn application_mp4() -> Self {
    z::bytes::Encoding::APPLICATION_MP4.into()
  }

  #[napi(factory)]
  pub fn application_soap_xml() -> Self {
    z::bytes::Encoding::APPLICATION_SOAP_XML.into()
  }

  #[napi(factory)]
  pub fn application_yang() -> Self {
    z::bytes::Encoding::APPLICATION_YANG.into()
  }

  #[napi(factory)]
  pub fn audio_aac() -> Self {
    z::bytes::Encoding::AUDIO_AAC.into()
  }

  #[napi(factory)]
  pub fn audio_flac() -> Self {
    z::bytes::Encoding::AUDIO_FLAC.into()
  }

  #[napi(factory)]
  pub fn audio_mp4() -> Self {
    z::bytes::Encoding::AUDIO_MP4.into()
  }

  #[napi(factory)]
  pub fn audio_ogg() -> Self {
    z::bytes::Encoding::AUDIO_OGG.into()
  }

  #[napi(factory)]
  pub fn audio_vorbis() -> Self {
    z::bytes::Encoding::AUDIO_VORBIS.into()
  }

  #[napi(factory)]
  pub fn video_h261() -> Self {
    z::bytes::Encoding::VIDEO_H261.into()
  }

  #[napi(factory)]
  pub fn video_h263() -> Self {
    z::bytes::Encoding::VIDEO_H263.into()
  }

  #[napi(factory)]
  pub fn video_h264() -> Self {
    z::bytes::Encoding::VIDEO_H264.into()
  }

  #[napi(factory)]
  pub fn video_h265() -> Self {
    z::bytes::Encoding::VIDEO_H265.into()
  }

  #[napi(factory)]
  pub fn video_h266() -> Self {
    z::bytes::Encoding::VIDEO_H266.into()
  }

  #[napi(factory)]
  pub fn video_mp4() -> Self {
    z::bytes::Encoding::VIDEO_MP4.into()
  }

  #[napi(factory)]
  pub fn video_ogg() -> Self {
    z::bytes::Encoding::VIDEO_OGG.into()
  }

  #[napi(factory)]
  pub fn video_raw() -> Self {
    z::bytes::Encoding::VIDEO_RAW.into()
  }

  #[napi(factory)]
  pub fn video_vp8() -> Self {
    z::bytes::Encoding::VIDEO_VP8.into()
  }

  #[napi(factory)]
  pub fn video_vp9() -> Self {
    z::bytes::Encoding::VIDEO_VP9.into()
  }

  // Exposed to JS as `toString()`; napi cannot surface a `Display` impl, so the
  // inherent method is deliberate despite clippy's `inherent_to_string` lint.
  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.0.to_string()
  }

  #[napi]
  pub fn with_schema(&self, value: String) -> Self {
    z::bytes::Encoding::from(self.0.to_string())
      .with_schema(value)
      .into()
  }
}
