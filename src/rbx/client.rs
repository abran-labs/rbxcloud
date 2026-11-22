//! Shared blocking transport for typed Open Cloud operations.
//!
//! The transport owns authentication, retries, response and error handling,
//! long-running operation polling and multipart uploads. It is not a request
//! builder: a [`Request`] can only be made by this crate's typed API modules,
//! each from a documented operation, and its host is one of the fixed
//! [`Server`] origins.
//!
//! [`Client`] blocks the calling thread. Async code, including every CLI
//! command, uses [`AsyncClient`], which runs the same transport on tokio's
//! blocking thread pool. Calling [`Client`] directly on an async runtime
//! thread returns [`Error::Config`] instead of stalling the runtime.
//!
//! Guarantees:
//!
//! - The credential is sent only to the operation's own server. Redirects are
//!   never followed; a redirect response is an [`Error::Redirect`].
//! - Production requests use HTTPS. Plain HTTP exists only for
//!   [`ClientPolicy::loopback`].
//! - Only reads, and writes explicitly marked idempotent, are retried, at
//!   most `max_retries` times. Operation polling stops after
//!   `max_poll_attempts` status checks.
//! - Time is bounded as well as counts: one attempt by `timeout`, one whole
//!   `execute` call by `max_total_time`, and every duration in the policy
//!   has a hard ceiling.
//! - Errors and returned JSON never contain the credential, and URLs inside
//!   errors have no query string.
mod async_client;
mod error;
// Typed API modules arrive in later tasks. Until then only the test fixtures
// call the request constructors.
#[cfg_attr(not(feature = "transport-fixtures"), allow(dead_code))]
mod operation;
mod policy;
mod redact;
#[cfg_attr(not(feature = "transport-fixtures"), allow(dead_code))]
mod request;
mod response;
mod send;

#[cfg(feature = "transport-fixtures")]
pub mod fixtures;

use std::thread;
use std::time::Instant;

use serde_json::Value;

use crate::rbx::credentials::Credentials;

pub use async_client::AsyncClient;
pub use error::{Error, TransportKind};
pub use policy::{
    ClientPolicy, Server, MAX_POLL_ATTEMPTS_LIMIT, MAX_POLL_DELAY_LIMIT, MAX_RETRIES_LIMIT,
    MAX_RETRY_DELAY_LIMIT, MAX_TIMEOUT_LIMIT, MAX_TOTAL_TIME_LIMIT,
};
pub use request::Request;

#[allow(unused_imports)]
pub(crate) use operation::{finished_operation, Method, Operation, Poll, PollFn};
#[allow(unused_imports)]
pub(crate) use request::Attachment;

use redact::Redactor;

/// A blocking Open Cloud client. Cheap to share by reference. In async code
/// use [`AsyncClient`].
pub struct Client {
    http: reqwest::blocking::Client,
    credentials: Credentials,
    policy: ClientPolicy,
    redactor: Redactor,
}

/// Prints the policy only.
impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("policy", &self.policy)
            .finish_non_exhaustive()
    }
}

impl Client {
    pub fn new(credentials: Credentials, policy: ClientPolicy) -> Result<Self, Error> {
        async_client::refuse_on_async_thread()?;
        policy.validate().map_err(Error::Config)?;
        if !credentials.secret().is_header_safe() {
            return Err(Error::Config(
                "the credential is empty or has characters that cannot be sent in an HTTP header"
                    .into(),
            ));
        }
        let redactor = Redactor::new(credentials.secret().clone());

        let mut builder = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(policy.timeout)
            .user_agent(concat!("rbxcloud/", env!("CARGO_PKG_VERSION")));
        builder = if policy.is_production() {
            builder.https_only(true)
        } else {
            // A loopback test server is never reached through a proxy.
            builder.no_proxy()
        };
        let http = builder
            .build()
            .map_err(|e| Error::Config(redactor.text(&e.without_url().to_string())))?;

        Ok(Self {
            http,
            credentials,
            policy,
            redactor,
        })
    }

    pub fn policy(&self) -> &ClientPolicy {
        &self.policy
    }

    /// Sends the request and returns the response body as JSON.
    ///
    /// An empty body (for example HTTP 204) is `Value::Null`; a `text/*` body
    /// is `Value::String`. If the request is a long-running operation, this
    /// waits for it and returns the finished operation.
    ///
    /// The whole call, with retries and status checks, is limited by
    /// [`ClientPolicy::max_total_time`].
    pub fn execute(&self, request: Request) -> Result<Value, Error> {
        async_client::refuse_on_async_thread()?;
        #[cfg(feature = "transport-fixtures")]
        fixtures::refuse_in_production(&request, &self.policy)?;

        let deadline = Instant::now() + self.policy.max_total_time;
        let mut value = self.send(&request, deadline, 0)?;
        let Some(poll) = request.poll else {
            return Ok(value);
        };

        let label = request.operation.label();
        let mut checks = 0u32;
        loop {
            let next = match poll(&value) {
                Ok(Poll::Ready) => return Ok(value),
                Ok(Poll::Failed(message)) => {
                    return Err(Error::OperationFailed {
                        operation: label,
                        message: self.redactor.text(&message),
                    })
                }
                Ok(Poll::Pending(next)) => next,
                Err(reason) => {
                    return Err(Error::InvalidResponse {
                        operation: label,
                        status: 200,
                        reason: self.redactor.text(&reason),
                    })
                }
            };
            if next.operation.method != Method::Get {
                return Err(Error::InvalidRequest {
                    operation: label,
                    reason: "an operation status check must be a GET".into(),
                });
            }
            if checks >= self.policy.max_poll_attempts {
                return Err(Error::OperationTimedOut {
                    operation: label,
                    attempts: checks,
                });
            }
            let left = deadline.saturating_duration_since(Instant::now());
            if self.policy.poll_delay >= left {
                return Err(Error::DeadlineExceeded {
                    operation: label,
                    limit: self.policy.max_total_time,
                    checks,
                });
            }
            thread::sleep(self.policy.poll_delay);
            checks += 1;
            value = self.send(&next, deadline, checks)?;
        }
    }
}
