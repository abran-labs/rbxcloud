//! Credential loading for the shared Open Cloud transport.
//!
//! A key is looked up in this order and the first usable value wins:
//!
//! 1. `ROBLOX_OPEN_CLOUD_API_KEY` in the environment
//! 2. `RBXCLOUD_API_KEY` in the environment (legacy name)
//! 3. the same two names, in the same order, in the credentials file
//!
//! The file is only opened when the environment has no key. It must be a
//! regular file, not a symbolic link, owned by the current user and readable
//! by nobody else (`chmod 600`).
//!
//! No error produced here contains a credential value, or any other text
//! taken from the environment value or the file contents.
mod env_file;
mod secret;

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

pub use secret::SecretString;

/// Preferred environment variable and credentials-file key.
pub const API_KEY_VAR: &str = "ROBLOX_OPEN_CLOUD_API_KEY";
/// Legacy name, used when [`API_KEY_VAR`] is unset or empty.
pub const LEGACY_API_KEY_VAR: &str = "RBXCLOUD_API_KEY";
/// Location of the credentials file, relative to the home directory.
pub const DEFAULT_FILE: &str = ".config/opencode/secrets/roblox-open-cloud.env";

const KEY_NAMES: [&str; 2] = [API_KEY_VAR, LEGACY_API_KEY_VAR];

/// How a request is authenticated.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Credentials {
    /// Open Cloud API key, sent as `x-api-key`.
    ApiKey(SecretString),
    /// OAuth 2.0 access token, sent as `Authorization: Bearer`. The caller
    /// supplies the token; nothing here obtains or refreshes one.
    Bearer(SecretString),
}

impl Credentials {
    pub fn api_key(key: SecretString) -> Self {
        Self::ApiKey(key)
    }

    pub fn bearer(token: SecretString) -> Self {
        Self::Bearer(token)
    }

    /// Finds the API key. `env` looks one variable up, so callers and tests
    /// choose the environment explicitly; see [`Credentials::from_process`]
    /// for the process environment and the default file.
    pub fn load(
        env: &dyn Fn(&str) -> Option<String>,
        credentials_file: &Path,
    ) -> Result<SecretString, CredentialsError> {
        match Self::load_env(env)? {
            Some(key) => Ok(key),
            None => Self::load_file(credentials_file),
        }
    }

    fn load_env(
        env: &dyn Fn(&str) -> Option<String>,
    ) -> Result<Option<SecretString>, CredentialsError> {
        for name in KEY_NAMES {
            if let Some(value) = env(name) {
                if let Some(key) = usable(value, name)? {
                    return Ok(Some(key));
                }
            }
        }

        Ok(None)
    }

    fn load_file(credentials_file: &Path) -> Result<SecretString, CredentialsError> {
        let Some(mut file) = env_file::read(credentials_file)? else {
            return Err(missing(credentials_file, false));
        };
        for name in KEY_NAMES {
            if let Some(value) = file.take(name) {
                let source = format!("{name} in {}", credentials_file.display());
                if let Some(key) = usable(value.into_inner(), &source)? {
                    return Ok(key);
                }
            }
        }
        Err(missing(credentials_file, true))
    }

    /// Loads the API key from the process environment, falling back to
    /// `~/.config/opencode/secrets/roblox-open-cloud.env`.
    pub fn from_process() -> Result<Self, CredentialsError> {
        for name in KEY_NAMES {
            if matches!(std::env::var(name), Err(std::env::VarError::NotUnicode(_))) {
                return Err(CredentialsError::InvalidValue {
                    source: name.to_string(),
                });
            }
        }
        if let Some(key) = Self::load_env(&|name| std::env::var(name).ok())? {
            return Ok(Self::ApiKey(key));
        }
        let file = Self::default_file().ok_or(CredentialsError::NoAbsoluteHome)?;
        Self::load_file(&file).map(Self::ApiKey)
    }

    /// The default credentials file, when an absolute home directory is known.
    pub fn default_file() -> Option<PathBuf> {
        let home =
            PathBuf::from(std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?);
        home.is_absolute().then(|| home.join(DEFAULT_FILE))
    }

    pub(crate) fn secret(&self) -> &SecretString {
        match self {
            Self::ApiKey(secret) | Self::Bearer(secret) => secret,
        }
    }
}

fn missing(file: &Path, file_present: bool) -> CredentialsError {
    CredentialsError::Missing {
        file: file.to_path_buf(),
        file_present,
    }
}

/// Trims the value and checks it can be sent as a header. Empty means unset.
fn usable(value: String, source: &str) -> Result<Option<SecretString>, CredentialsError> {
    let secret = SecretString::new(value);
    let trimmed = secret.expose_secret().trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let key = SecretString::new(trimmed.to_string());
    if !key.is_header_safe() {
        return Err(CredentialsError::InvalidValue {
            source: source.to_string(),
        });
    }
    Ok(Some(key))
}

/// Why no credential could be loaded. Never contains a credential value.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CredentialsError {
    /// File fallback is unavailable without an absolute home directory.
    NoAbsoluteHome,
    /// Neither the environment nor the credentials file has a key.
    Missing { file: PathBuf, file_present: bool },
    /// A value was found but cannot be sent as an HTTP header. `source` names
    /// where the value came from, never the value.
    InvalidValue { source: String },
    /// The credentials file exists but is not safe to trust.
    InsecureFile { file: PathBuf, reason: String },
    /// The credentials file is not `KEY=VALUE` lines. `line` is 1-based.
    MalformedFile {
        file: PathBuf,
        line: usize,
        reason: &'static str,
    },
    /// The credentials file could not be read.
    Unreadable { file: PathBuf, kind: io::ErrorKind },
}

impl fmt::Display for CredentialsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAbsoluteHome => write!(
                f,
                "no Roblox Open Cloud API key: set {API_KEY_VAR} (or {LEGACY_API_KEY_VAR}); \
                 credentials file fallback requires an absolute home directory"
            ),
            Self::Missing { file, file_present } => {
                let state = if *file_present {
                    "has neither key"
                } else {
                    "does not exist"
                };
                write!(
                    f,
                    "no Roblox Open Cloud API key: set {API_KEY_VAR} (or {LEGACY_API_KEY_VAR}); \
                     the credentials file {} {state}",
                    file.display()
                )
            }
            Self::InvalidValue { source } => write!(
                f,
                "the credential from {source} has characters that cannot be sent in an HTTP header"
            ),
            Self::InsecureFile { file, reason } => write!(
                f,
                "refusing to read credentials file {}: {reason}",
                file.display()
            ),
            Self::MalformedFile { file, line, reason } => write!(
                f,
                "credentials file {} line {line}: {reason}",
                file.display()
            ),
            Self::Unreadable { file, kind } => {
                write!(f, "cannot read credentials file {}: {kind}", file.display())
            }
        }
    }
}

impl std::error::Error for CredentialsError {}
