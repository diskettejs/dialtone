use napi_derive::napi;
use zenoh::bytes as zbytes;

#[napi]
pub struct Encoding {
  inner: zbytes::Encoding,
}

impl From<zbytes::Encoding> for Encoding {
  fn from(inner: zbytes::Encoding) -> Self {
    Encoding { inner }
  }
}

#[napi]
impl Encoding {
  #[napi(factory)]
  pub fn default() -> Self {
    Encoding {
      inner: zbytes::Encoding::default(),
    }
  }

  #[napi(factory)]
  pub fn from(value: String) -> Self {
    Encoding {
      inner: zbytes::Encoding::from(value),
    }
  }

  #[napi(factory)]
  pub fn zenoh_bytes() -> Self {
    zbytes::Encoding::ZENOH_BYTES.into()
  }

  #[napi(factory)]
  pub fn zenoh_string() -> Self {
    zbytes::Encoding::ZENOH_STRING.into()
  }

  #[napi(factory)]
  pub fn zenoh_serialized() -> Self {
    zbytes::Encoding::ZENOH_SERIALIZED.into()
  }

  #[napi(factory)]
  pub fn application_octet_stream() -> Self {
    zbytes::Encoding::APPLICATION_OCTET_STREAM.into()
  }

  #[napi(factory)]
  pub fn text_plain() -> Self {
    zbytes::Encoding::TEXT_PLAIN.into()
  }

  #[napi(factory)]
  pub fn application_json() -> Self {
    zbytes::Encoding::APPLICATION_JSON.into()
  }

  #[napi(factory)]
  pub fn text_json() -> Self {
    zbytes::Encoding::TEXT_JSON.into()
  }

  #[napi(factory)]
  pub fn application_cdr() -> Self {
    zbytes::Encoding::APPLICATION_CDR.into()
  }

  #[napi(factory)]
  pub fn application_cbor() -> Self {
    zbytes::Encoding::APPLICATION_CBOR.into()
  }

  #[napi(factory)]
  pub fn application_yaml() -> Self {
    zbytes::Encoding::APPLICATION_YAML.into()
  }

  #[napi(factory)]
  pub fn text_yaml() -> Self {
    zbytes::Encoding::TEXT_YAML.into()
  }

  #[napi(factory)]
  pub fn text_json5() -> Self {
    zbytes::Encoding::TEXT_JSON5.into()
  }

  #[napi(factory)]
  pub fn application_python_serialized_object() -> Self {
    zbytes::Encoding::APPLICATION_PYTHON_SERIALIZED_OBJECT.into()
  }

  #[napi(factory)]
  pub fn application_protobuf() -> Self {
    zbytes::Encoding::APPLICATION_PROTOBUF.into()
  }

  #[napi(factory)]
  pub fn application_java_serialized_object() -> Self {
    zbytes::Encoding::APPLICATION_JAVA_SERIALIZED_OBJECT.into()
  }

  #[napi(factory)]
  pub fn application_openmetrics_text() -> Self {
    zbytes::Encoding::APPLICATION_OPENMETRICS_TEXT.into()
  }

  #[napi(factory)]
  pub fn image_png() -> Self {
    zbytes::Encoding::IMAGE_PNG.into()
  }

  #[napi(factory)]
  pub fn image_jpeg() -> Self {
    zbytes::Encoding::IMAGE_JPEG.into()
  }

  #[napi(factory)]
  pub fn image_gif() -> Self {
    zbytes::Encoding::IMAGE_GIF.into()
  }

  #[napi(factory)]
  pub fn image_bmp() -> Self {
    zbytes::Encoding::IMAGE_BMP.into()
  }

  #[napi(factory)]
  pub fn image_webp() -> Self {
    zbytes::Encoding::IMAGE_WEBP.into()
  }

  #[napi(factory)]
  pub fn application_xml() -> Self {
    zbytes::Encoding::APPLICATION_XML.into()
  }

  #[napi(factory)]
  pub fn application_x_www_form_urlencoded() -> Self {
    zbytes::Encoding::APPLICATION_X_WWW_FORM_URLENCODED.into()
  }

  #[napi(factory)]
  pub fn text_html() -> Self {
    zbytes::Encoding::TEXT_HTML.into()
  }

  #[napi(factory)]
  pub fn text_xml() -> Self {
    zbytes::Encoding::TEXT_XML.into()
  }

  #[napi(factory)]
  pub fn text_css() -> Self {
    zbytes::Encoding::TEXT_CSS.into()
  }

  #[napi(factory)]
  pub fn text_javascript() -> Self {
    zbytes::Encoding::TEXT_JAVASCRIPT.into()
  }

  #[napi(factory)]
  pub fn text_markdown() -> Self {
    zbytes::Encoding::TEXT_MARKDOWN.into()
  }

  #[napi(factory)]
  pub fn text_csv() -> Self {
    zbytes::Encoding::TEXT_CSV.into()
  }

  #[napi(factory)]
  pub fn application_sql() -> Self {
    zbytes::Encoding::APPLICATION_SQL.into()
  }

  #[napi(factory)]
  pub fn application_coap_payload() -> Self {
    zbytes::Encoding::APPLICATION_COAP_PAYLOAD.into()
  }

  #[napi(factory)]
  pub fn application_json_patch_json() -> Self {
    zbytes::Encoding::APPLICATION_JSON_PATCH_JSON.into()
  }

  #[napi(factory)]
  pub fn application_json_seq() -> Self {
    zbytes::Encoding::APPLICATION_JSON_SEQ.into()
  }

  #[napi(factory)]
  pub fn application_jsonpath() -> Self {
    zbytes::Encoding::APPLICATION_JSONPATH.into()
  }

  #[napi(factory)]
  pub fn application_jwt() -> Self {
    zbytes::Encoding::APPLICATION_JWT.into()
  }

  #[napi(factory)]
  pub fn application_mp4() -> Self {
    zbytes::Encoding::APPLICATION_MP4.into()
  }

  #[napi(factory)]
  pub fn application_soap_xml() -> Self {
    zbytes::Encoding::APPLICATION_SOAP_XML.into()
  }

  #[napi(factory)]
  pub fn application_yang() -> Self {
    zbytes::Encoding::APPLICATION_YANG.into()
  }

  #[napi(factory)]
  pub fn audio_aac() -> Self {
    zbytes::Encoding::AUDIO_AAC.into()
  }

  #[napi(factory)]
  pub fn audio_flac() -> Self {
    zbytes::Encoding::AUDIO_FLAC.into()
  }

  #[napi(factory)]
  pub fn audio_mp4() -> Self {
    zbytes::Encoding::AUDIO_MP4.into()
  }

  #[napi(factory)]
  pub fn audio_ogg() -> Self {
    zbytes::Encoding::AUDIO_OGG.into()
  }

  #[napi(factory)]
  pub fn audio_vorbis() -> Self {
    zbytes::Encoding::AUDIO_VORBIS.into()
  }

  #[napi(factory)]
  pub fn video_h261() -> Self {
    zbytes::Encoding::VIDEO_H261.into()
  }

  #[napi(factory)]
  pub fn video_h263() -> Self {
    zbytes::Encoding::VIDEO_H263.into()
  }

  #[napi(factory)]
  pub fn video_h264() -> Self {
    zbytes::Encoding::VIDEO_H264.into()
  }

  #[napi(factory)]
  pub fn video_h265() -> Self {
    zbytes::Encoding::VIDEO_H265.into()
  }

  #[napi(factory)]
  pub fn video_h266() -> Self {
    zbytes::Encoding::VIDEO_H266.into()
  }

  #[napi(factory)]
  pub fn video_mp4() -> Self {
    zbytes::Encoding::VIDEO_MP4.into()
  }

  #[napi(factory)]
  pub fn video_ogg() -> Self {
    zbytes::Encoding::VIDEO_OGG.into()
  }

  #[napi(factory)]
  pub fn video_raw() -> Self {
    zbytes::Encoding::VIDEO_RAW.into()
  }

  #[napi(factory)]
  pub fn video_vp8() -> Self {
    zbytes::Encoding::VIDEO_VP8.into()
  }

  #[napi(factory)]
  pub fn video_vp9() -> Self {
    zbytes::Encoding::VIDEO_VP9.into()
  }

  // Exposed to JS as `toString()`; napi cannot surface a `Display` impl, so the
  // inherent method is deliberate despite clippy's `inherent_to_string` lint.
  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.inner.to_string()
  }

  #[napi]
  pub fn with_schema(&self, value: String) -> Self {
    Encoding {
      inner: zbytes::Encoding::from(self.inner.to_string()).with_schema(value),
    }
  }
}
