//! A hook that redirects requests to another hook.
//!
//! This is useful for forwarding requests to other hooks.
//!
//! # Example
//!
//! ```
//! use deboa::{DeboaRequest, DeboaResponse};
//! use deboa_extras::hook::RedirectHook;
//! use tackle::Hook;
//!
//! struct MyHook;
//!
//! impl Hook<DeboaRequest, DeboaResponse> for MyHook {
//!     type Result = Result<DeboaResponse>;
//!     type Error = deboa::errors::DeboaError;
//!
//!     async fn call(&self, input: DeboaRequest) -> Self::Result {
//!         Ok(DeboaResponse::new("Hello, world!"))
//!     }
//! }
//!
//! let hook = RedirectHook::new(MyHook);
//! ```
use deboa::{
    Result,
    errors::{DeboaError, RequestError},
    request::DeboaRequest,
    response::DeboaResponse,
};
use http::{StatusCode, Uri};
use hyper_body_utils::HttpBody;
use tackle::{Chain, Hook};

/// A hook that redirects requests to another hook.
pub struct RedirectHook<H> {
    inner: H,
    limit: usize,
}

impl<H> RedirectHook<H>
where
    H: Hook<DeboaRequest, DeboaResponse, Result = Result<DeboaResponse>, Error = DeboaError>,
{
    /// Creates a new redirect hook.
    pub fn new(inner: H, limit: usize) -> Self {
        Self { inner, limit }
    }
}

impl<H> Hook<DeboaRequest, DeboaResponse> for RedirectHook<H>
where
    H: Hook<DeboaRequest, DeboaResponse, Result = Result<DeboaResponse>, Error = DeboaError>,
{
    type Result = Result<DeboaResponse>;
    type Error = DeboaError;

    async fn call(&self, input: DeboaRequest) -> Self::Result {
        let request = input;
        let mut redirect_count = 0;
        let (parts, body) = request.into_parts();
        let parts_clone = parts.clone();
        let body_clone = body
            .try_clone()
            .unwrap_or_else(|_v| HttpBody::empty());

        let mut original_request = DeboaRequest::from_parts(parts_clone, body_clone)?;

        loop {
            let result = self
                .inner
                .call(original_request)
                .await?;

            if result
                .status()
                .is_redirection()
                && self.limit > 0
            {
                // Check if we should redirect
                if let Some(location) = result
                    .headers()
                    .get(http::header::LOCATION)
                {
                    if redirect_count >= self.limit {
                        return Err(DeboaError::Request(RequestError::Send {
                            message: "Redirect limit exceeded".to_string(),
                        }));
                    }

                    let location_str = location
                        .to_str()
                        .map_err(|_| DeboaError::Header {
                            message: "Invalid location header".to_string(),
                        })?;

                    let mut next_parts = parts.clone();
                    let next_body = if result.status() == StatusCode::PERMANENT_REDIRECT
                        || result.status() == StatusCode::TEMPORARY_REDIRECT
                    {
                        body.try_clone()
                            .unwrap_or_else(|_v| HttpBody::empty())
                    } else {
                        HttpBody::empty()
                    };

                    next_parts.uri = location_str
                        .parse::<Uri>()
                        .map_err(|e| {
                            DeboaError::Request(RequestError::UrlParse { message: e.to_string() })
                        })?;
                    original_request = DeboaRequest::from_parts(next_parts, next_body)?;
                    redirect_count += 1;
                } else {
                    return Ok(result);
                }
            } else {
                return Ok(result);
            }
        }
    }
}

/// A hook that redirects requests to a chain of hooks.
pub struct Redirect {
    limit: usize,
}

impl Default for Redirect {
    fn default() -> Self {
        Self { limit: 10 }
    }
}

impl Redirect {
    /// Creates a new redirect hook.
    pub fn new(limit: usize) -> Self {
        Self { limit }
    }
}

impl<H> Chain<H, DeboaError, DeboaRequest, DeboaResponse> for Redirect
where
    H: Hook<DeboaRequest, DeboaResponse, Result = Result<DeboaResponse>, Error = DeboaError>,
{
    type Hook = RedirectHook<H>;

    fn chain(&self, hook: H) -> Self::Hook {
        RedirectHook::new(hook, self.limit)
    }
}
