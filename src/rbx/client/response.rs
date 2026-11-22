//! Turns response bodies into JSON values and error messages.
use reqwest::header::{HeaderMap, CONTENT_TYPE};
use reqwest::StatusCode;
use serde_json::Value;

/// Parses a 2xx body. Empty is `Null`, `text/*` is a string, anything else
/// must be JSON.
pub(super) fn parse_success(headers: &HeaderMap, bytes: &[u8]) -> Result<Value, String> {
    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Ok(Value::Null);
    }
    let content_type = headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();
    if content_type.starts_with("text/") && !content_type.contains("json") {
        return String::from_utf8(bytes.to_vec())
            .map(Value::String)
            .map_err(|_| "the text body is not UTF-8".to_string());
    }
    // serde_json's message gives the position only, never the body text.
    serde_json::from_slice(bytes).map_err(|e| format!("the body is not valid JSON ({e})"))
}

/// The server's own error message, or the body text, or the status name.
pub(super) fn error_message(status: StatusCode, bytes: &[u8]) -> String {
    let fallback = || {
        status
            .canonical_reason()
            .unwrap_or("no error message")
            .to_string()
    };
    let text = String::from_utf8_lossy(bytes);
    let text = text.trim();
    if text.is_empty() {
        return fallback();
    }
    match serde_json::from_str::<Value>(text) {
        Ok(body) => json_message(&body).unwrap_or_else(|| text.to_string()),
        Err(_) => text.to_string(),
    }
}

/// Reads the error shapes Roblox uses:
/// `{"code","message"}`, `{"error","message"}`, `{"errors":[{"code","message"}]}`
/// and `{"error":{"code","message"}}`.
fn json_message(body: &Value) -> Option<String> {
    let label = |v: &Value| match v {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    };
    let entry = |v: &Value| -> Option<String> {
        let message = v.get("message").and_then(Value::as_str).map(str::to_string);
        let code = v
            .get("code")
            .and_then(label)
            .or_else(|| v.get("error").and_then(Value::as_str).map(str::to_string));
        match (code, message) {
            (Some(code), Some(message)) if !message.is_empty() => {
                Some(format!("{code}: {message}"))
            }
            (None, Some(message)) if !message.is_empty() => Some(message),
            (Some(code), _) => Some(code),
            _ => None,
        }
    };

    if let Some(errors) = body.get("errors").and_then(Value::as_array) {
        let messages: Vec<String> = errors.iter().filter_map(entry).collect();
        if !messages.is_empty() {
            return Some(messages.join("; "));
        }
    }
    if let Some(nested) = body.get("error").filter(|e| e.is_object()) {
        if let Some(message) = entry(nested) {
            return Some(message);
        }
    }
    entry(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(body: &str) -> String {
        error_message(StatusCode::BAD_REQUEST, body.as_bytes())
    }

    #[test]
    fn reads_each_roblox_error_shape() {
        assert_eq!(
            message(r#"{"code":"INVALID_ARGUMENT","message":"bad id"}"#),
            "INVALID_ARGUMENT: bad id"
        );
        assert_eq!(
            message(r#"{"error":"NOT_FOUND","message":"no entry","errorDetails":[]}"#),
            "NOT_FOUND: no entry"
        );
        assert_eq!(
            message(r#"{"errors":[{"code":0,"message":"first"},{"code":7,"message":"second"}]}"#),
            "0: first; 7: second"
        );
        assert_eq!(
            message(r#"{"error":{"code":"DENIED","message":"no scope"}}"#),
            "DENIED: no scope"
        );
        assert_eq!(message(r#"{"message":"only a message"}"#), "only a message");
    }

    #[test]
    fn falls_back_to_body_text_then_status_name() {
        assert_eq!(message("plain failure\n"), "plain failure");
        assert_eq!(message(r#"{"unknown":1}"#), r#"{"unknown":1}"#);
        assert_eq!(message("  "), "Bad Request");
    }

    #[test]
    fn parses_success_bodies_by_content_type() {
        let mut headers = HeaderMap::new();
        assert_eq!(parse_success(&headers, b"").unwrap(), Value::Null);
        assert_eq!(parse_success(&headers, b"{\"a\":1}").unwrap()["a"], 1);
        assert!(parse_success(&headers, b"not json").is_err());
        headers.insert(CONTENT_TYPE, "text/plain; charset=utf-8".parse().unwrap());
        assert_eq!(
            parse_success(&headers, b"123").unwrap(),
            Value::String("123".into())
        );
        headers.insert(CONTENT_TYPE, "text/json".parse().unwrap());
        assert_eq!(parse_success(&headers, b"123").unwrap(), 123);
    }
}
