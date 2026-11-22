//! Documented operations and long-running operation state.
//!
//! An [`Operation`] is a method, server and path template from the
//! specification, written as a constant by a typed API module.
use serde_json::Value;

use super::policy::Server;
use super::request::Request;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl Method {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
        }
    }
}

/// One documented operation: `path` is the template from the specification,
/// such as `/cloud/v2/groups/{group_id}`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Operation {
    pub(crate) method: Method,
    pub(crate) server: Server,
    pub(crate) path: &'static str,
    /// Whether a failed attempt may be sent again.
    pub(crate) retry: bool,
}

impl Operation {
    const fn new(method: Method, server: Server, path: &'static str, retry: bool) -> Self {
        Self {
            method,
            server,
            path,
            retry,
        }
    }

    /// A safe read. Retried on rate limits and transient failures.
    pub(crate) const fn get(server: Server, path: &'static str) -> Self {
        Self::new(Method::Get, server, path, true)
    }

    /// Not retried unless marked [`Operation::idempotent`].
    pub(crate) const fn post(server: Server, path: &'static str) -> Self {
        Self::new(Method::Post, server, path, false)
    }

    /// Not retried unless marked [`Operation::idempotent`].
    pub(crate) const fn put(server: Server, path: &'static str) -> Self {
        Self::new(Method::Put, server, path, false)
    }

    /// Not retried unless marked [`Operation::idempotent`].
    pub(crate) const fn patch(server: Server, path: &'static str) -> Self {
        Self::new(Method::Patch, server, path, false)
    }

    /// Not retried unless marked [`Operation::idempotent`].
    pub(crate) const fn delete(server: Server, path: &'static str) -> Self {
        Self::new(Method::Delete, server, path, false)
    }

    /// Marks a write as safe to send again. Use only when the Roblox
    /// documentation says a repeat has the same effect as one call.
    pub(crate) const fn idempotent(mut self) -> Self {
        self.retry = true;
        self
    }

    /// `METHOD /path/{template}`: the manifest id, used in errors.
    pub(crate) fn label(&self) -> String {
        format!("{} {}", self.method.as_str(), self.path)
    }
}

/// State of a long-running operation, read from a response by a [`PollFn`].
pub(crate) enum Poll {
    /// Finished; the response just read is the result.
    Ready,
    /// Finished with an error.
    Failed(String),
    /// Still running; send this request to check again.
    Pending(Box<Request>),
}

/// Reads a response and says what to do next. `Err` means the response does
/// not look like an operation at all.
pub(crate) type PollFn = fn(&Value) -> Result<Poll, String>;

/// The `done` and `error` fields shared by the Roblox operation resources.
/// Returns `Some(Ready | Failed)` when the operation is done, `None` while it
/// is still running.
pub(crate) fn finished_operation(operation: &Value) -> Option<Poll> {
    if operation.get("done").and_then(Value::as_bool) != Some(true) {
        return None;
    }
    Some(match operation.get("error").filter(|e| !e.is_null()) {
        None => Poll::Ready,
        Some(error) => {
            let code = error.get("code").map(|c| match c {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            });
            let message = error.get("message").and_then(Value::as_str);
            Poll::Failed(match (code, message) {
                (Some(code), Some(message)) => format!("{code}: {message}"),
                (None, Some(message)) => message.to_string(),
                (Some(code), None) => code,
                (None, None) => error.to_string(),
            })
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_reads_are_retried_unless_marked_idempotent() {
        let path = "/v1/x";
        assert!(Operation::get(Server::Apis, path).retry);
        assert!(!Operation::post(Server::Apis, path).retry);
        assert!(!Operation::put(Server::Apis, path).retry);
        assert!(!Operation::patch(Server::Apis, path).retry);
        assert!(!Operation::delete(Server::Apis, path).retry);
        assert!(Operation::put(Server::Apis, path).idempotent().retry);
    }

    #[test]
    fn reads_operation_state() {
        use serde_json::json;
        assert!(finished_operation(&json!({"path": "operations/1"})).is_none());
        assert!(finished_operation(&json!({"done": false})).is_none());
        assert!(matches!(
            finished_operation(&json!({"done": true, "error": null, "response": {}})),
            Some(Poll::Ready)
        ));
        let failed = finished_operation(
            &json!({"done": true, "error": {"code": "INVALID_ARGUMENT", "message": "bad file"}}),
        );
        assert!(matches!(failed, Some(Poll::Failed(m)) if m == "INVALID_ARGUMENT: bad file"));
        let numeric = finished_operation(&json!({"done": true, "error": {"code": 3}}));
        assert!(matches!(numeric, Some(Poll::Failed(m)) if m == "3"));
    }
}
