use std::fmt;

/// A credential value that never appears in `Debug` output.
///
/// There is no `Display`, `Serialize` or `PartialEq` implementation, so the
/// value only leaves through [`SecretString::expose_secret`].
#[derive(Clone)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// The raw value. Do not log or format it.
    pub fn expose_secret(&self) -> &str {
        &self.0
    }

    pub(crate) fn into_inner(self) -> String {
        self.0
    }

    /// Visible ASCII only: no spaces, control characters or line breaks, so
    /// the value cannot split or smuggle a header.
    pub(crate) fn is_header_safe(&self) -> bool {
        !self.0.is_empty() && self.0.bytes().all(|b| (0x21..=0x7e).contains(&b))
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretString([REDACTED])")
    }
}
