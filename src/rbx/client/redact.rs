//! Keeps credentials and signed query strings out of anything printable.
use serde_json::Value;

use crate::rbx::credentials::SecretString;

const MASK: &str = "[REDACTED]";
const MAX_TEXT: usize = 2000;
/// Bytes of masked text scanned for URLs: a generous multiple of what the
/// `MAX_TEXT` cap keeps.
const MAX_SCAN: usize = 8 * MAX_TEXT;

/// At most `max` bytes of `text`, cut on a character boundary.
fn prefix(text: &str, max: usize) -> &str {
    if text.len() <= max {
        return text;
    }
    let mut cut = max;
    while !text.is_char_boundary(cut) {
        cut -= 1;
    }
    &text[..cut]
}

pub(crate) struct Redactor {
    secret: SecretString,
}

impl Redactor {
    pub(crate) fn new(secret: SecretString) -> Self {
        Self { secret }
    }

    /// Masks the credential and the query string of every URL in `text`, then
    /// caps the length. Use for anything that ends up in an error.
    pub(crate) fn text(&self, text: &str) -> String {
        // Mask the credential over the whole text first, so a cut can never
        // leave part of it behind. Then cut before the URL scan, whose cost
        // grows faster than the text.
        let masked = self.secret_only(text);
        let scrubbed = strip_url_queries(prefix(&masked, MAX_SCAN));
        match scrubbed.char_indices().nth(MAX_TEXT) {
            Some((cut, _)) => format!("{}...", &scrubbed[..cut]),
            None => scrubbed,
        }
    }

    /// Masks the credential in every string of a response. Signed URLs are
    /// left alone: they are the payload the caller asked for.
    pub(crate) fn value(&self, value: &mut Value) {
        match value {
            Value::String(s) => {
                if s.contains(self.secret.expose_secret()) {
                    *s = self.secret_only(s);
                }
            }
            Value::Array(items) => items.iter_mut().for_each(|v| self.value(v)),
            Value::Object(map) => {
                let secret = self.secret.expose_secret();
                if map.keys().any(|k| k.contains(secret)) {
                    let entries = std::mem::take(map);
                    for (key, entry) in entries {
                        map.insert(self.secret_only(&key), entry);
                    }
                }
                map.values_mut().for_each(|v| self.value(v));
            }
            _ => {}
        }
    }

    fn secret_only(&self, text: &str) -> String {
        let secret = self.secret.expose_secret();
        if secret.is_empty() {
            return text.to_string();
        }
        text.replace(secret, MASK)
    }
}

/// `scheme://host/path` of a URL, dropping user info, query and fragment.
pub(crate) fn url_without_query(url: &reqwest::Url) -> String {
    let mut out = format!("{}://{}", url.scheme(), url.host_str().unwrap_or(""));
    if let Some(port) = url.port() {
        out.push_str(&format!(":{port}"));
    }
    out.push_str(url.path());
    out
}

/// Replaces `?query` and `#fragment` of every `http(s)://` URL in `text`.
fn strip_url_queries(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = find_url(rest) {
        let end = rest[start..]
            .find(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | '\\'))
            .map_or(rest.len(), |offset| start + offset);
        let url = &rest[start..end];
        out.push_str(&rest[..start]);
        match url.find(['?', '#']) {
            Some(cut) => {
                out.push_str(&url[..cut]);
                out.push_str(&url[cut..cut + 1]);
                out.push_str(MASK);
            }
            None => out.push_str(url),
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

fn find_url(text: &str) -> Option<usize> {
    let lower = text.to_ascii_lowercase();
    match (lower.find("http://"), lower.find("https://")) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn redactor() -> Redactor {
        Redactor::new(SecretString::new("s3cr3t-key"))
    }

    #[test]
    fn masks_secret_and_url_queries() {
        let out = redactor().text(
            "key s3cr3t-key at https://cdn.example/a/b?sig=abc&x=1 and HTTP://h/p#frag, done",
        );
        assert_eq!(
            out,
            "key [REDACTED] at https://cdn.example/a/b?[REDACTED] and HTTP://h/p#[REDACTED] done"
        );
    }

    #[test]
    fn leaves_plain_text_and_question_marks_alone() {
        assert_eq!(redactor().text("why? no url here"), "why? no url here");
        assert_eq!(redactor().text("see https://h/p."), "see https://h/p.");
    }

    #[test]
    fn caps_length_on_a_character_boundary() {
        let out = redactor().text(&"é".repeat(MAX_TEXT + 50));
        assert_eq!(out.chars().count(), MAX_TEXT + 3);
    }

    #[test]
    fn huge_text_is_cut_before_the_url_scan_without_leaking_a_partial_secret() {
        // The secret straddles the scan cut; many URLs would make an uncut
        // scan very slow.
        let mut text = "x".repeat(MAX_SCAN - 4);
        text.push_str("s3cr3t-key ");
        text.push_str(&"http://h/p?q=1 ".repeat(200_000));
        let started = std::time::Instant::now();
        let out = redactor().text(&text);
        assert!(started.elapsed() < std::time::Duration::from_secs(2));
        assert_eq!(out.chars().count(), MAX_TEXT + 3);
        assert!(!redactor().text(&text[MAX_SCAN - 100..]).contains("s3cr"));
    }

    #[test]
    fn masks_secret_in_nested_values_and_keys() {
        let mut value = serde_json::json!({
            "a": ["x s3cr3t-key y", 1],
            "s3cr3t-key": {"url": "https://h/p?sig=keep"}
        });
        redactor().value(&mut value);
        assert_eq!(
            value,
            serde_json::json!({
                "a": ["x [REDACTED] y", 1],
                "[REDACTED]": {"url": "https://h/p?sig=keep"}
            })
        );
    }
}
