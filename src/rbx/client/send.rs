//! Sends one request, with bounded retries.
use std::io::Read;
use std::thread;
use std::time::Duration;

use reqwest::blocking::multipart::{Form, Part as FormPart};
use reqwest::blocking::RequestBuilder;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, LOCATION};
use serde_json::Value;

use super::error::{Error, TransportKind};
use super::operation::Method;
use super::redact::url_without_query;
use super::request::{Body, Part, Request};
use super::response;
use super::Client;
use crate::rbx::credentials::Credentials;

/// Headers the transport sets itself. A typed request may not override them.
const RESERVED_HEADERS: [&str; 9] = [
    "authorization",
    "x-api-key",
    "cookie",
    "host",
    "content-length",
    "content-type",
    "transfer-encoding",
    "connection",
    "proxy-authorization",
];

enum Failure {
    /// Worth another attempt if the operation allows it.
    Retry {
        error: Error,
        wait: Option<Duration>,
    },
    Fatal(Error),
}

impl Client {
    /// Sends `request`, retrying only if its operation is marked retryable.
    pub(super) fn send(&self, request: &Request) -> Result<Value, Error> {
        let label = request.operation.label();
        let invalid = |reason: String| Error::InvalidRequest {
            operation: label.clone(),
            reason,
        };
        let origin = self.policy.origin(request.operation.server);
        let url = request.url(&origin).map_err(&invalid)?;
        check_headers(request).map_err(&invalid)?;

        let retries = if request.operation.retry {
            self.policy.max_retries
        } else {
            0
        };
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            let (error, wait) = match self.attempt(request, &url, &label, attempt) {
                Ok(value) => return Ok(value),
                Err(Failure::Fatal(error)) => return Err(error),
                Err(Failure::Retry { error, wait }) => (error, wait),
            };
            if attempt > retries {
                return Err(error);
            }
            let backoff = self
                .policy
                .retry_delay
                .saturating_mul(1u32 << (attempt - 1).min(16));
            thread::sleep(wait.unwrap_or(backoff).min(self.policy.max_retry_delay));
        }
    }

    fn attempt(
        &self,
        request: &Request,
        url: &reqwest::Url,
        label: &str,
        attempt: u32,
    ) -> Result<Value, Failure> {
        let builder = self.build(request, url).map_err(|reason| {
            Failure::Fatal(Error::InvalidRequest {
                operation: label.to_string(),
                reason,
            })
        })?;

        let response = builder.send().map_err(|e| {
            let kind = if e.is_timeout() {
                TransportKind::Timeout
            } else if e.is_connect() {
                TransportKind::Connect
            } else {
                TransportKind::Other
            };
            Failure::Retry {
                error: Error::Transport {
                    operation: label.to_string(),
                    kind,
                    detail: self.redactor.text(&error_chain(&e.without_url())),
                    attempts: attempt,
                },
                wait: None,
            }
        })?;

        let status = response.status();
        let headers = response.headers().clone();

        if status.is_redirection() {
            let location = headers
                .get(LOCATION)
                .and_then(|v| v.to_str().ok())
                .map(|raw| match url.join(raw) {
                    Ok(target) => url_without_query(&target),
                    Err(_) => self.redactor.text(raw),
                })
                .map(|target| self.redactor.text(&target));
            return Err(Failure::Fatal(Error::Redirect {
                operation: label.to_string(),
                status: status.as_u16(),
                location,
            }));
        }

        let limit = self.policy.max_response_bytes;
        let mut bytes = Vec::new();
        let read = response
            .take(limit.saturating_add(1))
            .read_to_end(&mut bytes);
        if let Err(e) = read {
            return Err(Failure::Retry {
                error: Error::Transport {
                    operation: label.to_string(),
                    kind: TransportKind::Other,
                    detail: self
                        .redactor
                        .text(&format!("reading the response failed: {e}")),
                    attempts: attempt,
                },
                wait: None,
            });
        }
        if bytes.len() as u64 > limit {
            return Err(Failure::Fatal(Error::InvalidResponse {
                operation: label.to_string(),
                status: status.as_u16(),
                reason: format!("the body is larger than {limit} bytes"),
            }));
        }

        if status.is_success() {
            let mut value = response::parse_success(&headers, &bytes).map_err(|reason| {
                Failure::Fatal(Error::InvalidResponse {
                    operation: label.to_string(),
                    status: status.as_u16(),
                    reason: self.redactor.text(&reason),
                })
            })?;
            self.redactor.value(&mut value);
            return Ok(value);
        }

        let error = Error::Status {
            operation: label.to_string(),
            status: status.as_u16(),
            message: self.redactor.text(&response::error_message(status, &bytes)),
            attempts: attempt,
        };
        if matches!(status.as_u16(), 429 | 502 | 503 | 504) {
            Err(Failure::Retry {
                error,
                wait: retry_after(&headers),
            })
        } else {
            Err(Failure::Fatal(error))
        }
    }

    fn build(&self, request: &Request, url: &reqwest::Url) -> Result<RequestBuilder, String> {
        let method = match request.operation.method {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Patch => reqwest::Method::PATCH,
            Method::Delete => reqwest::Method::DELETE,
        };
        let mut builder = self.http.request(method, url.clone());

        for (name, value) in &request.headers {
            let value = HeaderValue::from_str(value)
                .map_err(|_| format!("header {name} has a value that cannot be sent"))?;
            builder = builder.header(*name, value);
        }

        let (name, text) = match &self.credentials {
            Credentials::ApiKey(key) => ("x-api-key", key.expose_secret().to_string()),
            Credentials::Bearer(token) => (
                AUTHORIZATION.as_str(),
                format!("Bearer {}", token.expose_secret()),
            ),
        };
        let mut credential = HeaderValue::from_str(&text)
            .map_err(|_| "the credential cannot be sent in an HTTP header".to_string())?;
        credential.set_sensitive(true);
        builder = builder.header(name, credential);

        Ok(match &request.body {
            Body::None => builder,
            Body::Json(data) => builder
                .header(CONTENT_TYPE, "application/json")
                .body(data.to_vec()),
            Body::Multipart(parts) => {
                let mut form = Form::new();
                for (name, part) in parts {
                    let part = match part {
                        Part::Text(text) => FormPart::text(text.clone()),
                        Part::File(file) => FormPart::bytes(file.data.to_vec())
                            .file_name(file.file_name.clone())
                            .mime_str(&file.content_type)
                            .map_err(|_| {
                                format!("attachment {name} has an invalid content type")
                            })?,
                    };
                    form = form.part(*name, part);
                }
                builder.multipart(form)
            }
        })
    }
}

fn check_headers(request: &Request) -> Result<(), String> {
    for (name, _) in &request.headers {
        if RESERVED_HEADERS.contains(&name.to_ascii_lowercase().as_str()) {
            return Err(format!("header {name} is set by the transport"));
        }
    }
    Ok(())
}

/// Seconds to wait, from `Retry-After` or Roblox's `x-ratelimit-reset`.
/// An HTTP-date `Retry-After` is ignored and the normal backoff applies.
fn retry_after(headers: &HeaderMap) -> Option<Duration> {
    ["retry-after", "x-ratelimit-reset"]
        .iter()
        .find_map(|name| {
            let seconds: f64 = headers.get(*name)?.to_str().ok()?.trim().parse().ok()?;
            (seconds.is_finite() && seconds >= 0.0)
                .then(|| Duration::from_secs_f64(seconds.min(86_400.0)))
        })
}

/// The error and its causes on one line. `reqwest` only prints the outermost.
fn error_chain(error: &dyn std::error::Error) -> String {
    let mut text = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        text.push_str(": ");
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    text
}
