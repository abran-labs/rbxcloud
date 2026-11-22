//! Reads and parses the credentials file.
//!
//! Reading checks that the file is safe to trust. Parsing only splits
//! `KEY=VALUE` lines: every value goes straight into a [`SecretString`] and
//! parse errors carry a line number, never text from the file.
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

use super::{CredentialsError, SecretString};

/// A credentials file is a few short lines; anything larger is not one.
const MAX_BYTES: u64 = 64 * 1024;

pub(super) struct EnvFile {
    entries: Vec<(String, SecretString)>,
}

impl EnvFile {
    pub(super) fn take(&mut self, name: &str) -> Option<SecretString> {
        let at = self.entries.iter().position(|(n, _)| n == name)?;
        Some(self.entries.swap_remove(at).1)
    }
}

/// Returns `Ok(None)` when the file does not exist.
pub(super) fn read(path: &Path) -> Result<Option<EnvFile>, CredentialsError> {
    let insecure = |reason: String| CredentialsError::InsecureFile {
        file: path.to_path_buf(),
        reason,
    };
    let unreadable = |e: io::Error| CredentialsError::Unreadable {
        file: path.to_path_buf(),
        kind: e.kind(),
    };

    // Look before opening so a symbolic link, directory, FIFO or device is
    // refused without being opened.
    let listed = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(unreadable(e)),
    };
    if listed.file_type().is_symlink() {
        return Err(insecure("it is a symbolic link".into()));
    }
    if !listed.is_file() {
        return Err(insecure("it is not a regular file".into()));
    }

    let file = open(path).map_err(|e| {
        if is_symlink_refusal(&e) {
            insecure("it is a symbolic link".into())
        } else {
            unreadable(e)
        }
    })?;
    // Checks below use the open handle, so they describe the bytes that are
    // read even if the path is replaced in the meantime.
    let opened = file.metadata().map_err(unreadable)?;
    if !opened.is_file() {
        return Err(insecure("it is not a regular file".into()));
    }
    check_access(&opened).map_err(insecure)?;
    if opened.len() > MAX_BYTES {
        return Err(insecure(format!("it is larger than {MAX_BYTES} bytes")));
    }

    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(unreadable)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(insecure(format!("it is larger than {MAX_BYTES} bytes")));
    }
    let text = String::from_utf8(bytes).map_err(|_| CredentialsError::MalformedFile {
        file: path.to_path_buf(),
        line: 0,
        reason: "the file is not UTF-8 text",
    })?;

    parse(&text)
        .map(Some)
        .map_err(|(line, reason)| CredentialsError::MalformedFile {
            file: path.to_path_buf(),
            line,
            reason,
        })
}

#[cfg(unix)]
fn open(path: &Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .read(true)
        // O_NOFOLLOW: fail if the last path component became a link after the
        // check above. O_NONBLOCK: never hang if it became a FIFO.
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
}

#[cfg(not(unix))]
fn open(path: &Path) -> io::Result<File> {
    File::open(path)
}

#[cfg(unix)]
fn is_symlink_refusal(e: &io::Error) -> bool {
    e.raw_os_error() == Some(libc::ELOOP)
}

#[cfg(not(unix))]
fn is_symlink_refusal(_: &io::Error) -> bool {
    false
}

#[cfg(unix)]
fn check_access(meta: &fs::Metadata) -> Result<(), String> {
    use std::os::unix::fs::MetadataExt;
    let mode = meta.mode() & 0o7777;
    if mode & 0o077 != 0 {
        return Err(format!(
            "its mode is {mode:04o}, which lets other users read or change it; run chmod 600"
        ));
    }
    // SAFETY: geteuid takes no arguments, has no preconditions and cannot fail.
    let user = unsafe { libc::geteuid() };
    if meta.uid() != user {
        return Err("it is not owned by the current user".into());
    }
    Ok(())
}

/// Windows has no mode bits; the file's ACL is not inspected.
#[cfg(not(unix))]
fn check_access(_: &fs::Metadata) -> Result<(), String> {
    Ok(())
}

/// Splits `KEY=VALUE` lines. Blank lines and `#` comments are skipped, an
/// `export ` prefix is allowed, and a value may be wrapped in one pair of
/// matching quotes. Nothing is expanded or unescaped.
fn parse(text: &str) -> Result<EnvFile, (usize, &'static str)> {
    let mut entries: Vec<(String, SecretString)> = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let number = index + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = match line.strip_prefix("export") {
            Some(rest) if rest.starts_with([' ', '\t']) => rest.trim_start(),
            _ => line,
        };
        let Some((name, value)) = line.split_once('=') else {
            return Err((number, "expected KEY=VALUE"));
        };
        let name = name.trim_end();
        let valid_start = name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_');
        if !valid_start || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err((number, "the key is not a valid variable name"));
        }
        if entries.iter().any(|(existing, _)| existing == name) {
            return Err((number, "the key is defined more than once"));
        }
        let value = unquote(value.trim()).ok_or((number, "the value has an unmatched quote"))?;
        entries.push((name.to_string(), SecretString::new(value)));
    }
    Ok(EnvFile { entries })
}

fn unquote(value: &str) -> Option<&str> {
    for quote in ['"', '\''] {
        if let Some(rest) = value.strip_prefix(quote) {
            return rest.strip_suffix(quote).filter(|v| !v.contains(quote));
        }
    }
    let unquoted = !value.ends_with(['"', '\'']);
    unquoted.then_some(value)
}
