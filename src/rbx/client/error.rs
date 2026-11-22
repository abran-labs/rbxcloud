use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::rbx::credentials::CredentialsError;

/// How a request failed before any HTTP response arrived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    Connect,
    Timeout,
    Other,
}

/// A shared-transport failure.
///
/// Every text field has been through the client's redaction: it never holds
/// the credential, and URLs in it have no query string. `operation` is the
/// documented `METHOD /path/{template}`, not the filled-in URL.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// No usable credential.
    Credentials(CredentialsError),
    /// The [`ClientPolicy`](super::ClientPolicy) or HTTP client is invalid.
    Config(String),
    /// The request could not be built, so nothing was sent.
    InvalidRequest { operation: String, reason: String },
    /// A file attachment could not be read.
    File { path: PathBuf, kind: io::ErrorKind },
    /// No HTTP response arrived. `attempts` counts every try.
    Transport {
        operation: String,
        kind: TransportKind,
        detail: String,
        attempts: u32,
    },
    /// The server answered with a non-success status. `message` is the
    /// server's own error message when it sent one.
    Status {
        operation: String,
        status: u16,
        message: String,
        attempts: u32,
    },
    /// The server answered with a redirect. Redirects are never followed, so
    /// the credential is never sent to the redirect target.
    Redirect {
        operation: String,
        status: u16,
        location: Option<String>,
    },
    /// The response was not the JSON this client expects.
    InvalidResponse {
        operation: String,
        status: u16,
        reason: String,
    },
    /// A long-running operation finished with an error.
    OperationFailed { operation: String, message: String },
    /// A long-running operation was still running after the last allowed
    /// status check. `attempts` is the number of status checks made.
    OperationTimedOut { operation: String, attempts: u32 },
}

impl Error {
    /// HTTP status, when the server answered.
    pub fn status(&self) -> Option<u16> {
        match self {
            Self::Status { status, .. }
            | Self::Redirect { status, .. }
            | Self::InvalidResponse { status, .. } => Some(*status),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Credentials(e) => write!(f, "{e}"),
            Self::Config(reason) => write!(f, "invalid client configuration: {reason}"),
            Self::InvalidRequest { operation, reason } => {
                write!(f, "{operation}: invalid request: {reason}")
            }
            Self::File { path, kind } => {
                write!(f, "cannot read attachment {}: {kind}", path.display())
            }
            Self::Transport {
                operation,
                kind,
                detail,
                attempts,
            } => {
                let what = match kind {
                    TransportKind::Connect => "connection failed",
                    TransportKind::Timeout => "timed out",
                    TransportKind::Other => "request failed",
                };
                write!(
                    f,
                    "{operation}: {what} after {attempts} attempt(s): {detail}"
                )
            }
            Self::Status {
                operation,
                status,
                message,
                attempts,
            } => {
                write!(f, "{operation}: HTTP {status}: {message}")?;
                if *attempts > 1 {
                    write!(f, " (after {attempts} attempts)")?;
                }
                Ok(())
            }
            Self::Redirect {
                operation,
                status,
                location,
            } => {
                let target = location.as_deref().unwrap_or("an unspecified location");
                write!(
                    f,
                    "{operation}: HTTP {status}: refused to follow redirect to {target}"
                )
            }
            Self::InvalidResponse {
                operation,
                status,
                reason,
            } => write!(
                f,
                "{operation}: HTTP {status}: unexpected response: {reason}"
            ),
            Self::OperationFailed { operation, message } => {
                write!(f, "{operation}: operation failed: {message}")
            }
            Self::OperationTimedOut {
                operation,
                attempts,
            } => write!(
                f,
                "{operation}: operation still running after {attempts} status check(s)"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Credentials(e) => Some(e),
            _ => None,
        }
    }
}

impl From<CredentialsError> for Error {
    fn from(e: CredentialsError) -> Self {
        Self::Credentials(e)
    }
}

/// Lets code that returns the legacy [`rbx::error::Error`](crate::rbx::error::Error)
/// use the shared transport with `?`. The legacy enum is unchanged.
impl From<Error> for crate::rbx::error::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::Status {
                status, message, ..
            } => Self::HttpStatusError {
                code: status,
                msg: message,
            },
            Error::File { path, kind } => {
                Self::FileLoadError(format!("{}: {kind}", path.display()))
            }
            other => Self::EndpointError(other.to_string()),
        }
    }
}
