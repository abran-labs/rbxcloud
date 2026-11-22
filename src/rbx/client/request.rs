//! Typed request data.
//!
//! A [`Request`] fills in the path template of a documented [`Operation`].
//! Both are built with `pub(crate)` functions only: code outside this crate
//! can pass a `Request` to the client but cannot make one, so the library has
//! no arbitrary method/path/body entry point.
use std::fmt;
use std::io;
use std::path::Path;
use std::sync::Arc;

use reqwest::Url;
use serde::Serialize;

use super::error::Error;
use super::operation::{Operation, PollFn};

/// A file part of a multipart request.
#[derive(Clone)]
pub(crate) struct Attachment {
    pub(crate) file_name: String,
    pub(crate) content_type: String,
    pub(crate) data: Arc<[u8]>,
}

impl Attachment {
    /// Fails if `file_name` is not a plain file name; see
    /// [`check_file_name`].
    pub(crate) fn from_bytes(
        file_name: impl Into<String>,
        content_type: impl Into<String>,
        data: Vec<u8>,
    ) -> Result<Self, Error> {
        let file_name = file_name.into();
        check_file_name(&file_name).map_err(|reason| Error::InvalidAttachment {
            reason: reason.to_string(),
        })?;
        Ok(Self {
            file_name,
            content_type: content_type.into(),
            data: data.into(),
        })
    }

    /// Reads the file now, so a retry sends the same bytes. The part is named
    /// after the last component of `path`.
    pub(crate) fn from_path(path: &Path, content_type: impl Into<String>) -> Result<Self, Error> {
        let file_error = |kind| Error::File {
            path: path.to_path_buf(),
            kind,
        };
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| file_error(io::ErrorKind::InvalidInput))?
            .to_string();
        let data = std::fs::read(path).map_err(|e| file_error(e.kind()))?;
        Self::from_bytes(file_name, content_type, data)
    }
}

/// Longest attachment file name, in bytes.
pub(crate) const MAX_FILE_NAME_BYTES: usize = 255;

/// Checks a name that is written into a multipart part header as
/// `filename="..."`. A line break there would start a new part header, and
/// servers disagree on how quotes and backslashes are unescaped, so the name
/// must be a plain file name: not empty, not `.` or `..`, at most
/// [`MAX_FILE_NAME_BYTES`] bytes, and without control characters, line or
/// paragraph separators, `"`, `/` or `\`.
pub(crate) fn check_file_name(name: &str) -> Result<(), &'static str> {
    if name.is_empty() || name.trim().is_empty() {
        return Err("the file name is empty");
    }
    if name == "." || name == ".." {
        return Err("the file name is a dot segment");
    }
    if name.len() > MAX_FILE_NAME_BYTES {
        return Err("the file name is longer than 255 bytes");
    }
    if name
        .chars()
        .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
    {
        return Err("the file name has a control character or line break");
    }
    if name.contains(['"', '/', '\\']) {
        return Err("the file name has a quote or path separator");
    }
    Ok(())
}

pub(crate) enum Part {
    Text(String),
    File(Attachment),
}

pub(crate) enum Body {
    None,
    Json(Arc<[u8]>),
    Multipart(Vec<(&'static str, Part)>),
}

/// One call of a documented operation. Made only by this crate's typed API
/// modules; run with [`Client::execute`](super::Client::execute).
pub struct Request {
    pub(crate) operation: Operation,
    path_params: Vec<(&'static str, String)>,
    query: Vec<(&'static str, String)>,
    pub(crate) headers: Vec<(&'static str, String)>,
    pub(crate) body: Body,
    pub(crate) poll: Option<PollFn>,
}

/// Prints the operation only: parameters and bodies may hold secrets.
impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Request({})", self.operation.label())
    }
}

impl Request {
    pub(crate) fn new(operation: Operation) -> Self {
        Self {
            operation,
            path_params: Vec::new(),
            query: Vec::new(),
            headers: Vec::new(),
            body: Body::None,
            poll: None,
        }
    }

    /// Fills `{name}` in the path template. The value is percent-encoded as
    /// one path segment, so it cannot add segments or a query string.
    pub(crate) fn path_param(mut self, name: &'static str, value: impl fmt::Display) -> Self {
        self.path_params.push((name, value.to_string()));
        self
    }

    /// Adds a query parameter. Call again with the same name for a repeated
    /// (array) parameter.
    pub(crate) fn query(mut self, name: &'static str, value: impl fmt::Display) -> Self {
        self.query.push((name, value.to_string()));
        self
    }

    pub(crate) fn query_opt(self, name: &'static str, value: Option<impl fmt::Display>) -> Self {
        match value {
            Some(value) => self.query(name, value),
            None => self,
        }
    }

    /// Adds a documented request header. Authentication and framing headers
    /// are refused when the request is sent.
    pub(crate) fn header(mut self, name: &'static str, value: impl fmt::Display) -> Self {
        self.headers.push((name, value.to_string()));
        self
    }

    pub(crate) fn header_opt(self, name: &'static str, value: Option<impl fmt::Display>) -> Self {
        match value {
            Some(value) => self.header(name, value),
            None => self,
        }
    }

    /// Sends `body` as `application/json`.
    pub(crate) fn json<T: Serialize + ?Sized>(mut self, body: &T) -> Result<Self, Error> {
        let bytes = serde_json::to_vec(body).map_err(|e| Error::InvalidRequest {
            operation: self.operation.label(),
            reason: format!("the body cannot be written as JSON ({:?})", e.classify()),
        })?;
        self.body = Body::Json(bytes.into());
        Ok(self)
    }

    /// Adds a text field to a `multipart/form-data` body.
    pub(crate) fn multipart_text(self, name: &'static str, value: impl Into<String>) -> Self {
        self.push_part(name, Part::Text(value.into()))
    }

    /// Adds a file to a `multipart/form-data` body.
    pub(crate) fn multipart_file(self, name: &'static str, file: Attachment) -> Self {
        self.push_part(name, Part::File(file))
    }

    fn push_part(mut self, name: &'static str, part: Part) -> Self {
        match &mut self.body {
            Body::Multipart(parts) => parts.push((name, part)),
            _ => self.body = Body::Multipart(vec![(name, part)]),
        }
        self
    }

    /// Treats the response as a long-running operation and waits for it,
    /// within the policy's poll limits.
    pub(crate) fn poll_with(mut self, poll: PollFn) -> Self {
        self.poll = Some(poll);
        self
    }

    /// Builds the URL under `origin`. Fails if the template and the supplied
    /// parameters do not match exactly.
    pub(crate) fn url(&self, origin: &Url) -> Result<Url, String> {
        let template = self.operation.path;
        if !template.starts_with('/') {
            return Err("the path template must start with '/'".into());
        }
        let mut path = String::with_capacity(template.len() + 16);
        let mut used = 0usize;
        let mut rest = template;
        while let Some(open) = rest.find('{') {
            let close = rest[open..]
                .find('}')
                .map(|offset| open + offset)
                .ok_or("the path template has an unclosed '{'")?;
            let name = &rest[open + 1..close];
            let mut matches = self.path_params.iter().filter(|(n, _)| *n == name);
            let (_, value) = matches
                .next()
                .ok_or_else(|| format!("path parameter {name} is missing"))?;
            if matches.next().is_some() {
                return Err(format!("path parameter {name} is set more than once"));
            }
            // "." and ".." would be resolved as directory steps by the URL
            // parser, even when percent-encoded.
            if value.is_empty() || value == "." || value == ".." {
                return Err(format!("path parameter {name} is empty or a dot segment"));
            }
            path.push_str(&rest[..open]);
            encode_segment(value, &mut path);
            used += 1;
            rest = &rest[close + 1..];
        }
        if rest.contains('}') {
            return Err("the path template has an unopened '}'".into());
        }
        path.push_str(rest);
        if used != self.path_params.len() {
            return Err("a path parameter is not part of the path template".into());
        }
        if path.contains(['?', '#']) {
            return Err("the path template must not hold a query or fragment".into());
        }

        let mut url = origin.clone();
        url.set_path(&path);
        if !self.query.is_empty() {
            url.query_pairs_mut().extend_pairs(self.query.iter());
        }
        // The parser must not have changed where the request goes.
        if url.path() != path || url.origin() != origin.origin() {
            return Err("the path does not survive URL normalisation".into());
        }
        Ok(url)
    }
}

/// Percent-encodes everything except RFC 3986 unreserved characters.
fn encode_segment(value: &str, out: &mut String) {
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rbx::client::policy::Server;

    const ITEM: Operation = Operation::get(Server::Apis, "/v1/things/{thing_id}/parts/{part_id}");

    fn origin() -> Url {
        Url::parse("https://apis.roblox.com").unwrap()
    }

    #[test]
    fn fills_and_encodes_path_parameters() {
        let url = Request::new(ITEM)
            .path_param("thing_id", 42)
            .path_param("part_id", "a/b?c#d %é")
            .query("filter", "x y&z=1")
            .query("tag", "one")
            .query("tag", "two")
            .url(&origin())
            .unwrap();
        assert_eq!(
            url.as_str(),
            "https://apis.roblox.com/v1/things/42/parts/a%2Fb%3Fc%23d%20%25%C3%A9\
             ?filter=x+y%26z%3D1&tag=one&tag=two"
        );
    }

    #[test]
    fn rejects_parameters_that_do_not_match_the_template() {
        let build = |r: Request| r.url(&origin()).unwrap_err();
        let missing = build(Request::new(ITEM).path_param("thing_id", 1));
        assert!(missing.contains("part_id is missing"), "{missing}");
        let extra = build(
            Request::new(ITEM)
                .path_param("thing_id", 1)
                .path_param("part_id", 2)
                .path_param("other", 3),
        );
        assert!(extra.contains("not part of the path template"), "{extra}");
        let twice = build(
            Request::new(ITEM)
                .path_param("thing_id", 1)
                .path_param("thing_id", 2)
                .path_param("part_id", 3),
        );
        assert!(twice.contains("more than once"), "{twice}");
    }

    #[test]
    fn rejects_values_that_would_change_the_path() {
        for value in ["", ".", ".."] {
            let error = Request::new(ITEM)
                .path_param("thing_id", value)
                .path_param("part_id", 1)
                .url(&origin())
                .unwrap_err();
            assert!(error.contains("dot segment"), "{value:?}: {error}");
        }
    }

    #[test]
    fn a_value_cannot_move_the_request_to_another_host() {
        let url = Request::new(ITEM)
            .path_param("thing_id", "@evil.test")
            .path_param("part_id", "//evil.test/x")
            .url(&origin())
            .unwrap();
        assert_eq!(url.host_str(), Some("apis.roblox.com"));
        assert_eq!(
            url.path(),
            "/v1/things/%40evil.test/parts/%2F%2Fevil.test%2Fx"
        );
    }

    #[test]
    fn file_names_must_be_plain() {
        for good in [
            "a.png",
            "pic ture.png",
            "Tëst ☕.fbx",
            "x",
            &"n".repeat(255),
        ] {
            assert!(check_file_name(good).is_ok(), "{good:?}");
            assert!(Attachment::from_bytes(good, "image/png", vec![1]).is_ok());
        }
        let long = "n".repeat(256);
        let bad = [
            "",
            "   ",
            ".",
            "..",
            long.as_str(),
            "a\r\nContent-Type: text/html\r\n\r\nx.png",
            "a\nb.png",
            "a\rb.png",
            "a\tb.png",
            "a\0b.png",
            "a\u{7f}b.png",
            "a\u{85}b.png",
            "a\u{2028}b.png",
            "a\u{2029}b.png",
            "a\"b.png",
            "dir/a.png",
            "dir\\a.png",
        ];
        for name in bad {
            assert!(check_file_name(name).is_err(), "{name:?}");
            let error = Attachment::from_bytes(name, "image/png", vec![1])
                .err()
                .unwrap();
            let text = format!("{error} | {error:?}");
            assert!(matches!(error, Error::InvalidAttachment { .. }), "{name:?}");
            assert!(
                !text.contains("text/html"),
                "the name is not echoed: {text}"
            );
        }
    }

    #[test]
    fn rejects_malformed_templates() {
        for path in ["v1/x", "/v1/{open", "/v1/close}", "/v1/x?y=1", "/v1/../x"] {
            let operation = Operation::get(Server::Apis, path);
            assert!(Request::new(operation).url(&origin()).is_err(), "{path}");
        }
    }
}
