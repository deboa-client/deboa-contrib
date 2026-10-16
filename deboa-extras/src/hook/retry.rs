//! A module that contains hooks for retrying requests.
//! This is useful for retrying requests that fail due to transient errors.
//!
//! # Example
//! ```
//! use deboa::{DeboaRequest, DeboaResponse};
//! use deboa_extras::hook::Retry;
//! use tackle::Hook;
//!
//!```
use crate::hook::retry::strategy::FixedIntervalRetryStrategy;
use deboa::{
    Result,
    errors::{DeboaError, RequestError},
    request::DeboaRequest,
    response::DeboaResponse,
};
use http::StatusCode;
use hyper_body_utils::HttpBody;
use std::time::Duration;
use tackle::{Chain, Hook};

/// A trait that defines a retry strategy.
pub trait RetryStrategy: Clone + Copy {
    /// Returns the maximum number of retries.
    fn limit(&self) -> usize;
    /// Returns the current attempt number.
    fn attempt(&self) -> usize;
    /// Returns the next retry delay, or None if the limit has been reached.
    fn next_retry(&mut self) -> Option<Duration>;
}

/// A trait that defines a sleeper.
pub trait Sleeper: Clone + Send {
    /// Sleeps for the given duration.
    fn sleep(&self, duration: Duration) -> impl std::future::Future<Output = ()> + Send;
}

/// A hook that redirects requests to another hook.
pub struct RetryHook<H, S, Z> {
    inner: H,
    strategy: S,
    sleeper: Z,
}

impl<H, S, Z> RetryHook<H, S, Z>
where
    H: Hook<DeboaRequest, DeboaResponse, Result = Result<DeboaResponse>, Error = DeboaError>,
    S: RetryStrategy + Send,
    Z: Sleeper,
{
    /// Creates a new redirect hook.
    pub fn new(inner: H, strategy: S, sleeper: Z) -> Self {
        Self { inner, strategy, sleeper }
    }
}

impl<H, S, Z> Hook<DeboaRequest, DeboaResponse> for RetryHook<H, S, Z>
where
    H: Hook<DeboaRequest, DeboaResponse, Result = Result<DeboaResponse>, Error = DeboaError>,
    S: RetryStrategy + Send,
    Z: Sleeper,
{
    type Result = Result<DeboaResponse>;
    type Error = DeboaError;

    async fn call(&self, request: DeboaRequest) -> Self::Result {
        let (parts, body) = request.into_parts();
        let mut strategy = self.strategy;
        while let Some(delay) = strategy.next_retry() {
            log::info!("Attempt {} of {}", strategy.attempt(), strategy.limit());
            let parts_clone = parts.clone();
            let body_clone = body
                .try_clone()
                .unwrap_or_else(|_v| HttpBody::empty());

            log::info!("Retrying request to {} after {:?}...", parts_clone.uri, delay);

            let next_request = DeboaRequest::from_parts(parts_clone, body_clone)?;

            let response = self
                .inner
                .call(next_request)
                .await?;

            log::info!("Response status: {}", response.status());

            if response.status() != StatusCode::SERVICE_UNAVAILABLE
                && response.status() != StatusCode::GATEWAY_TIMEOUT
                && response.status() != StatusCode::BAD_GATEWAY
                && response.status() != StatusCode::TOO_MANY_REQUESTS
                && response.status() != StatusCode::INTERNAL_SERVER_ERROR
            {
                return Ok(response);
            }

            log::info!(
                "Request {} failed with status code {}. Retrying in {:?}...",
                strategy.attempt(),
                response.status(),
                delay
            );

            self.sleeper
                .sleep(delay)
                .await;
        }
        Err(DeboaError::Request(RequestError::Send {
            message: format!("Could not complete request after {} retries", strategy.attempt()),
        }))
    }
}

/// A hook that redirects requests to a chain of hooks.
pub struct Retry<Z, S = FixedIntervalRetryStrategy> {
    strategy: S,
    sleeper: Z,
}

impl<Z> Default for Retry<Z>
where
    Z: Sleeper + Default + Copy,
{
    fn default() -> Self {
        Self {
            strategy: FixedIntervalRetryStrategy::new(3, Duration::from_secs(3)),
            sleeper: Z::default(),
        }
    }
}

impl<Z, S> Retry<Z, S> {
    /// Creates a new redirect hook.
    pub fn new(strategy: S, sleeper: Z) -> Self {
        Self { strategy, sleeper }
    }
}

impl<H, S, Z> Chain<H, DeboaError, DeboaRequest, DeboaResponse> for Retry<Z, S>
where
    H: Hook<DeboaRequest, DeboaResponse, Result = Result<DeboaResponse>, Error = DeboaError>,
    S: RetryStrategy + Send,
    Z: Sleeper + Copy,
{
    type Hook = RetryHook<H, S, Z>;

    fn chain(&self, hook: H) -> Self::Hook {
        RetryHook::new(hook, self.strategy, self.sleeper)
    }
}

/// A module that contains retry strategies.
pub mod strategy {
    use crate::hook::retry::RetryStrategy;
    use std::time::Duration;

    #[derive(Debug, Clone, Copy)]
    /// A fixed interval retry strategy that retries a fixed number of times with a fixed delay.
    pub struct FixedIntervalRetryStrategy {
        limit: usize,
        attempt: usize,
        delay: Duration,
    }

    impl FixedIntervalRetryStrategy {
        /// Creates a new fixed interval retry strategy.
        pub fn new(limit: usize, delay: Duration) -> Self {
            Self { limit, attempt: 0, delay }
        }
    }

    impl RetryStrategy for FixedIntervalRetryStrategy {
        fn limit(&self) -> usize {
            self.limit
        }

        fn attempt(&self) -> usize {
            self.attempt
        }

        fn next_retry(&mut self) -> Option<Duration> {
            if self.attempt < self.limit {
                self.attempt += 1;
                Some(self.delay)
            } else {
                None
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    /// A linear retry strategy that retries a fixed number of times with a fixed delay.
    pub struct LinearRetryStrategy {
        limit: usize,
        attempt: usize,
        delay: Duration,
    }

    impl LinearRetryStrategy {
        /// Creates a new linear retry strategy.
        pub fn new(limit: usize, delay: Duration) -> Self {
            Self { limit, attempt: 0, delay }
        }
    }

    impl RetryStrategy for LinearRetryStrategy {
        fn limit(&self) -> usize {
            self.limit
        }

        fn attempt(&self) -> usize {
            self.attempt
        }

        fn next_retry(&mut self) -> Option<Duration> {
            if self.attempt < self.limit {
                self.attempt += 1;
                Some(self.delay * self.attempt as u32)
            } else {
                None
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    /// A exponential backoff retry strategy that retries a fixed number of times with a fixed delay.
    pub struct ExponentialBackoffRetryStrategy {
        limit: usize,
        attempt: usize,
        delay: Duration,
    }

    impl ExponentialBackoffRetryStrategy {
        /// Creates a new exponential backoff retry strategy.
        pub fn new(limit: usize, delay: Duration) -> Self {
            Self { limit, attempt: 0, delay }
        }
    }

    impl RetryStrategy for ExponentialBackoffRetryStrategy {
        fn limit(&self) -> usize {
            self.limit
        }

        fn attempt(&self) -> usize {
            self.attempt
        }

        fn next_retry(&mut self) -> Option<Duration> {
            if self.attempt < self.limit {
                self.attempt += 1;
                Some(self.delay * 2u32.pow(self.attempt as u32 - 1))
            } else {
                None
            }
        }
    }
}
