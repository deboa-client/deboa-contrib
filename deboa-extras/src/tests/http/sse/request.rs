use crate::http::sse::request::ServerSentEventBuilder;
use deboa::{Result, request::DeboaRequestBuilder};
use http::{HeaderValue, Method, Uri, header};
use mime_typed::{MimeStrExt, TextEventStream};

#[test]
fn test_sse_request() -> Result<()> {
    let request = DeboaRequestBuilder::sse("https://sse.dev/test")?.build()?;
    assert_eq!(request.method(), Method::GET);
    assert_eq!(request.uri(), &Uri::from_static("https://sse.dev/test"));
    assert_eq!(
        request
            .headers()
            .get(header::CONTENT_TYPE),
        Some(HeaderValue::from_static(TextEventStream::MIME_STR)).as_ref()
    );
    Ok(())
}
