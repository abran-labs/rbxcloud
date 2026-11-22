//! Shared transport tests. Every request goes to a scripted server on a
//! loopback `TcpListener`; nothing here reaches the network.
mod support;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use rbxcloud::rbx::client::{
    fixtures, AsyncClient, Client, ClientPolicy, Error, Server, TransportKind,
};
use rbxcloud::rbx::credentials::{Credentials, CredentialsError, SecretString};
use serde_json::{json, Value};
use support::{parse_multipart, Reply, TestServer};

const KEY: &str = "test-key-0123456789abcdef";
const FILE_KEY: &str = "file-key-fedcba9876543210";

// ---------------------------------------------------------------- helpers

fn env_of(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let map: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    move |name: &str| map.get(name).cloned()
}

fn no_env() -> impl Fn(&str) -> Option<String> {
    env_of(&[])
}

fn write_secret_file(dir: &Path, name: &str, contents: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, contents).unwrap();
    set_mode(&path, 0o600);
    path
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

#[cfg(not(unix))]
fn set_mode(_: &Path, _: u32) {}

fn policy(server: &TestServer) -> ClientPolicy {
    ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(5))
        .retry_delay(Duration::from_millis(1))
        .poll_delay(Duration::from_millis(1))
}

fn client_with(policy: ClientPolicy) -> Client {
    Client::new(Credentials::api_key(SecretString::new(KEY)), policy).unwrap()
}

fn client(server: &TestServer) -> Client {
    client_with(policy(server))
}

/// Fails if `text` contains the credential.
fn assert_no_secret(text: &str) {
    assert!(!text.contains(KEY), "credential leaked: {text}");
    assert!(!text.contains(FILE_KEY), "credential leaked: {text}");
}

fn both_formats(error: &dyn std::error::Error) -> String {
    format!("{error} | {error:?}")
}

// ------------------------------------------------------------ credentials

#[test]
fn env_prefers_the_roblox_name_over_the_legacy_name() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_secret_file(dir.path(), "c.env", "ROBLOX_OPEN_CLOUD_API_KEY=from-file\n");

    let both = env_of(&[
        ("RBXCLOUD_API_KEY", "legacy-value"),
        ("ROBLOX_OPEN_CLOUD_API_KEY", "preferred-value"),
    ]);
    let key = Credentials::load(&both, &file).unwrap();
    assert_eq!(key.expose_secret(), "preferred-value");

    let legacy = env_of(&[("RBXCLOUD_API_KEY", " legacy-value\n")]);
    let key = Credentials::load(&legacy, &file).unwrap();
    assert_eq!(key.expose_secret(), "legacy-value", "value is trimmed");

    let blank_preferred = env_of(&[
        ("ROBLOX_OPEN_CLOUD_API_KEY", "  "),
        ("RBXCLOUD_API_KEY", "legacy-value"),
    ]);
    let key = Credentials::load(&blank_preferred, &file).unwrap();
    assert_eq!(key.expose_secret(), "legacy-value", "blank counts as unset");
}

#[test]
fn env_key_means_the_file_is_never_opened() {
    let dir = tempfile::tempdir().unwrap();
    // This file would be rejected (malformed, wrong mode) if it were read.
    let file = dir.path().join("bad.env");
    fs::write(&file, "not a key value line\n").unwrap();
    set_mode(&file, 0o644);

    let env = env_of(&[("ROBLOX_OPEN_CLOUD_API_KEY", KEY)]);
    assert_eq!(Credentials::load(&env, &file).unwrap().expose_secret(), KEY);
}

#[test]
fn file_fallback_reads_the_key_and_ignores_other_entries() {
    let dir = tempfile::tempdir().unwrap();
    let contents = format!(
        "# Roblox Open Cloud key\n\n\
         ROBLOX_CREATOR_USER_ID=12345\n\
         export RBXCLOUD_API_KEY='legacy-in-file'\n\
         ROBLOX_OPEN_CLOUD_API_KEY=\"{FILE_KEY}\"\r\n"
    );
    let file = write_secret_file(dir.path(), "c.env", &contents);
    let key = Credentials::load(&no_env(), &file).unwrap();
    assert_eq!(
        key.expose_secret(),
        FILE_KEY,
        "preferred name wins in the file"
    );

    let legacy_only = write_secret_file(dir.path(), "l.env", "RBXCLOUD_API_KEY = legacy-in-file\n");
    let key = Credentials::load(&no_env(), &legacy_only).unwrap();
    assert_eq!(key.expose_secret(), "legacy-in-file");

    // A value that holds '=' (base64 padding) is kept whole.
    let padded = write_secret_file(dir.path(), "p.env", "ROBLOX_OPEN_CLOUD_API_KEY=abc==\n");
    let key = Credentials::load(&no_env(), &padded).unwrap();
    assert_eq!(key.expose_secret(), "abc==");
}

#[test]
fn missing_key_is_reported_without_reading_anything_secret() {
    let dir = tempfile::tempdir().unwrap();

    let absent = dir.path().join("absent.env");
    let error = Credentials::load(&no_env(), &absent).unwrap_err();
    assert!(matches!(
        &error,
        CredentialsError::Missing { file_present: false, file } if file == &absent
    ));
    let text = error.to_string();
    assert!(text.contains("ROBLOX_OPEN_CLOUD_API_KEY"), "{text}");
    assert!(text.contains("RBXCLOUD_API_KEY"), "{text}");
    assert!(text.contains("does not exist"), "{text}");

    let other = write_secret_file(dir.path(), "o.env", "ROBLOX_CREATOR_USER_ID=12345\n");
    let error = Credentials::load(&no_env(), &other).unwrap_err();
    assert!(matches!(
        error,
        CredentialsError::Missing {
            file_present: true,
            ..
        }
    ));

    let blank = write_secret_file(dir.path(), "b.env", "ROBLOX_OPEN_CLOUD_API_KEY=\n");
    let error = Credentials::load(&no_env(), &blank).unwrap_err();
    assert!(matches!(
        error,
        CredentialsError::Missing {
            file_present: true,
            ..
        }
    ));
}

#[test]
fn malformed_files_are_rejected_without_echoing_their_contents() {
    let dir = tempfile::tempdir().unwrap();
    let cases: [(&str, String, usize); 5] = [
        ("no equals sign", format!("# comment\n{FILE_KEY}\n"), 2),
        ("bad key name", format!("ROBLOX OPEN={FILE_KEY}\n"), 1),
        (
            "unmatched quote",
            format!("ROBLOX_OPEN_CLOUD_API_KEY=\"{FILE_KEY}\n"),
            1,
        ),
        (
            "duplicate key",
            format!("ROBLOX_OPEN_CLOUD_API_KEY={FILE_KEY}\nROBLOX_OPEN_CLOUD_API_KEY=other\n"),
            2,
        ),
        ("stray quote", format!("RBXCLOUD_API_KEY={FILE_KEY}'\n"), 1),
    ];
    for (name, contents, expected_line) in cases {
        let file = write_secret_file(dir.path(), "m.env", &contents);
        let error = Credentials::load(&no_env(), &file).unwrap_err();
        assert!(
            matches!(&error, CredentialsError::MalformedFile { line, .. } if *line == expected_line),
            "{name}: {error:?}"
        );
        assert_no_secret(&both_formats(&error));
    }

    let binary = dir.path().join("bin.env");
    fs::write(&binary, [0xff, 0xfe, b'=', 0x00]).unwrap();
    set_mode(&binary, 0o600);
    let error = Credentials::load(&no_env(), &binary).unwrap_err();
    assert!(
        matches!(error, CredentialsError::MalformedFile { .. }),
        "{error:?}"
    );
}

#[test]
fn values_that_cannot_be_headers_are_rejected_without_being_printed() {
    let dir = tempfile::tempdir().unwrap();
    let absent = dir.path().join("absent.env");

    let env = env_of(&[(
        "ROBLOX_OPEN_CLOUD_API_KEY",
        "test-key-0123456789abcdef\r\nX-Evil: 1",
    )]);
    let error = Credentials::load(&env, &absent).unwrap_err();
    assert!(
        matches!(&error, CredentialsError::InvalidValue { source } if source == "ROBLOX_OPEN_CLOUD_API_KEY"),
        "{error:?}"
    );
    assert_no_secret(&both_formats(&error));

    let file = write_secret_file(
        dir.path(),
        "s.env",
        &format!("ROBLOX_OPEN_CLOUD_API_KEY={FILE_KEY} trailing words\n"),
    );
    let error = Credentials::load(&no_env(), &file).unwrap_err();
    assert!(
        matches!(error, CredentialsError::InvalidValue { .. }),
        "{error:?}"
    );
    assert_no_secret(&both_formats(&error));
}

#[cfg(unix)]
#[test]
fn files_readable_by_others_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_secret_file(
        dir.path(),
        "c.env",
        &format!("ROBLOX_OPEN_CLOUD_API_KEY={FILE_KEY}\n"),
    );
    for mode in [0o644, 0o640, 0o604, 0o660, 0o602, 0o610] {
        set_mode(&file, mode);
        let error = Credentials::load(&no_env(), &file).unwrap_err();
        assert!(
            matches!(&error, CredentialsError::InsecureFile { reason, .. } if reason.contains("chmod 600")),
            "mode {mode:o}: {error:?}"
        );
        assert_no_secret(&both_formats(&error));
    }
    for mode in [0o600, 0o400] {
        set_mode(&file, mode);
        let key = Credentials::load(&no_env(), &file).unwrap();
        assert_eq!(key.expose_secret(), FILE_KEY, "mode {mode:o}");
    }
}

#[cfg(unix)]
#[test]
fn symlinks_and_non_files_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let real = write_secret_file(
        dir.path(),
        "real.env",
        &format!("ROBLOX_OPEN_CLOUD_API_KEY={FILE_KEY}\n"),
    );
    let link = dir.path().join("link.env");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let error = Credentials::load(&no_env(), &link).unwrap_err();
    assert!(
        matches!(&error, CredentialsError::InsecureFile { reason, .. } if reason.contains("symbolic link")),
        "{error:?}"
    );

    let dangling = dir.path().join("dangling.env");
    std::os::unix::fs::symlink(dir.path().join("nowhere"), &dangling).unwrap();
    let error = Credentials::load(&no_env(), &dangling).unwrap_err();
    assert!(
        matches!(error, CredentialsError::InsecureFile { .. }),
        "{error:?}"
    );

    let error = Credentials::load(&no_env(), dir.path()).unwrap_err();
    assert!(
        matches!(&error, CredentialsError::InsecureFile { reason, .. } if reason.contains("regular file")),
        "{error:?}"
    );
}

#[test]
fn oversized_files_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let contents = format!(
        "ROBLOX_OPEN_CLOUD_API_KEY={FILE_KEY}\n# {}\n",
        "x".repeat(70_000)
    );
    let file = write_secret_file(dir.path(), "big.env", &contents);
    let error = Credentials::load(&no_env(), &file).unwrap_err();
    assert!(
        matches!(error, CredentialsError::InsecureFile { .. }),
        "{error:?}"
    );
}

#[test]
fn secrets_are_redacted_in_debug_output() {
    let secret = SecretString::new(KEY);
    assert_eq!(format!("{secret:?}"), "SecretString([REDACTED])");
    assert_no_secret(&format!("{:?}", Credentials::api_key(secret.clone())));
    assert_no_secret(&format!("{:#?}", Credentials::bearer(secret)));

    let server = TestServer::start(vec![]);
    let client = client(&server);
    assert_no_secret(&format!("{client:?}"));
    assert_no_secret(&format!("{:?}", fixtures::get_item(KEY)));
}

// ------------------------------------------------------------------ policy

#[test]
fn default_policy_targets_the_roblox_servers_over_https() {
    let policy = ClientPolicy::default();
    assert!(policy.is_production());
    assert_eq!(
        policy.origin(Server::Apis).as_str(),
        "https://apis.roblox.com/"
    );
    assert_eq!(
        policy.origin(Server::ItemConfiguration).as_str(),
        "https://itemconfiguration.roblox.com/"
    );
    assert_eq!(
        policy.origin(Server::Thumbnails).as_str(),
        "https://thumbnails.roblox.com/"
    );
}

#[test]
fn server_origins_match_the_coverage_manifest() {
    let manifest = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/openapi/coverage.json"
    ))
    .expect("read openapi/coverage.json");
    let manifest: Value = serde_json::from_str(&manifest).unwrap();
    let mut in_manifest: Vec<&str> = manifest["operations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|op| op["server"].as_str().unwrap())
        .collect();
    in_manifest.sort_unstable();
    in_manifest.dedup();
    let mut known: Vec<&str> = Server::ALL.iter().map(|s| s.origin()).collect();
    known.sort_unstable();
    assert_eq!(known, in_manifest);
}

#[test]
fn only_loopback_addresses_can_replace_a_server() {
    assert!(ClientPolicy::loopback("127.0.0.1:8080".parse().unwrap()).is_ok());
    assert!(ClientPolicy::loopback("[::1]:8080".parse().unwrap()).is_ok());
    for addr in [
        "10.0.0.5:80",
        "93.184.216.34:443",
        "0.0.0.0:80",
        "[2001:db8::1]:80",
    ] {
        let error = ClientPolicy::loopback(addr.parse().unwrap()).unwrap_err();
        assert!(error.contains("not a loopback address"), "{addr}: {error}");
    }

    let local = ClientPolicy::loopback("127.0.0.1:1".parse().unwrap()).unwrap();
    assert!(!local.is_production());
    assert!(local
        .clone()
        .with_loopback_server(Server::Thumbnails, "10.0.0.5:80".parse().unwrap())
        .is_err());
    assert!(ClientPolicy::default()
        .with_loopback_server(Server::Thumbnails, "127.0.0.1:1".parse().unwrap())
        .is_err());
}

#[test]
fn out_of_range_policies_and_credentials_are_rejected() {
    let new = |policy| Client::new(Credentials::api_key(SecretString::new(KEY)), policy);
    assert!(matches!(
        new(ClientPolicy::default().max_retries(11)),
        Err(Error::Config(_))
    ));
    assert!(matches!(
        new(ClientPolicy::default().max_poll_attempts(1001)),
        Err(Error::Config(_))
    ));
    assert!(matches!(
        new(ClientPolicy::default().timeout(Duration::ZERO)),
        Err(Error::Config(_))
    ));
    assert!(new(ClientPolicy::default()).is_ok());

    for bad in ["", "has space", "line\nbreak"] {
        let result = Client::new(
            Credentials::api_key(SecretString::new(bad)),
            ClientPolicy::default(),
        );
        assert!(matches!(result, Err(Error::Config(_))), "{bad:?}");
    }
}

#[test]
fn fixtures_are_never_sent_to_production() {
    // No network: the request is refused before a URL is built.
    let client = client_with(ClientPolicy::default());
    let error = client.execute(fixtures::get_item("1")).unwrap_err();
    assert!(matches!(error, Error::InvalidRequest { .. }), "{error:?}");
}

// ---------------------------------------------------------------- requests

#[test]
fn get_sends_the_key_path_and_query() {
    let server = TestServer::start(vec![
        Reply::json(200, r#"{"id":"a b","n":1}"#),
        Reply::json(200, r#"{"items":[]}"#),
    ]);
    let client = client(&server);

    let item = client.execute(fixtures::get_item("a b/c?d")).unwrap();
    assert_eq!(item, json!({"id": "a b", "n": 1}));
    let list = client
        .execute(fixtures::list_items(
            Some(25),
            Some("tok en&x=1"),
            &["red", "blue"],
        ))
        .unwrap();
    assert_eq!(list, json!({"items": []}));

    let seen = server.requests();
    assert_eq!(seen.len(), 2);
    assert_eq!(seen[0].method, "GET");
    assert_eq!(seen[0].target, "/transport-fixtures/items/a%20b%2Fc%3Fd");
    assert_eq!(seen[0].header("x-api-key"), Some(KEY));
    assert_eq!(seen[0].header("authorization"), None);
    assert_eq!(seen[0].header("content-type"), None);
    assert!(matches!(seen[0].header("content-length"), None | Some("0")));
    assert!(seen[0]
        .header("user-agent")
        .unwrap()
        .starts_with("rbxcloud/"));
    assert!(seen[0].body.is_empty());
    assert_eq!(
        seen[1].target,
        "/transport-fixtures/items?maxPageSize=25&pageToken=tok+en%26x%3D1&tag=red&tag=blue"
    );
}

#[test]
fn bearer_credentials_use_the_authorization_header() {
    let server = TestServer::start(vec![Reply::json(200, "{}")]);
    let client = Client::new(Credentials::bearer(SecretString::new(KEY)), policy(&server)).unwrap();
    client.execute(fixtures::get_item("1")).unwrap();
    let seen = server.requests();
    assert_eq!(
        seen[0].header("authorization"),
        Some(format!("Bearer {KEY}").as_str())
    );
    assert_eq!(seen[0].header("x-api-key"), None);
}

#[test]
fn json_bodies_headers_and_methods_are_sent_as_typed() {
    let server = TestServer::start(vec![
        Reply::json(200, r#"{"created":true}"#),
        Reply::json(200, r#"{"updated":true}"#),
    ]);
    let client = client(&server);
    let body = json!({"displayName": "Café ☕", "count": 3, "nested": {"ok": true}});

    client.execute(fixtures::create_item(&body)).unwrap();
    client
        .execute(fixtures::update_item("7", "displayName,count", &body))
        .unwrap();

    let seen = server.requests();
    assert_eq!(seen[0].method, "POST");
    assert_eq!(seen[0].target, "/transport-fixtures/items");
    assert_eq!(seen[0].header("content-type"), Some("application/json"));
    assert_eq!(
        serde_json::from_slice::<Value>(&seen[0].body).unwrap(),
        body
    );
    assert_eq!(seen[1].method, "PATCH");
    assert_eq!(
        seen[1].target,
        "/transport-fixtures/items/7?updateMask=displayName%2Ccount"
    );
    assert_eq!(seen[1].header("roblox-place-id"), Some("99"));
    assert_eq!(seen[1].header("x-api-key"), Some(KEY));
    assert_eq!(
        serde_json::from_slice::<Value>(&seen[1].body).unwrap(),
        body
    );
}

#[test]
fn a_typed_request_cannot_override_transport_headers() {
    let server = TestServer::start(vec![]);
    let error = client(&server)
        .execute(fixtures::get_item_overriding_api_key("1", "attacker-key"))
        .unwrap_err();
    assert!(matches!(error, Error::InvalidRequest { .. }), "{error:?}");
    assert_eq!(server.connections(), 0, "nothing was sent");
}

#[test]
fn dot_segment_path_values_are_refused_before_sending() {
    let server = TestServer::start(vec![]);
    let client = client(&server);
    for value in ["..", ".", ""] {
        let error = client.execute(fixtures::get_item(value)).unwrap_err();
        assert!(
            matches!(error, Error::InvalidRequest { .. }),
            "{value:?}: {error:?}"
        );
    }
    assert_eq!(server.connections(), 0);
}

#[test]
fn each_operation_goes_to_its_own_server() {
    let apis = TestServer::start(vec![Reply::json(200, r#"{"from":"apis"}"#)]);
    let thumbnails = TestServer::start(vec![Reply::json(200, r#"{"from":"thumbnails"}"#)]);
    let items = TestServer::start(vec![Reply::json(200, r#"{"from":"itemconfiguration"}"#)]);
    let policy = policy(&apis)
        .with_loopback_server(Server::Thumbnails, thumbnails.addr())
        .unwrap()
        .with_loopback_server(Server::ItemConfiguration, items.addr())
        .unwrap();
    let client = client_with(policy);

    assert_eq!(
        client.execute(fixtures::get_item("1")).unwrap()["from"],
        "apis"
    );
    assert_eq!(
        client.execute(fixtures::get_thumbnail(2)).unwrap()["from"],
        "thumbnails"
    );
    assert_eq!(
        client.execute(fixtures::get_collectible(3)).unwrap()["from"],
        "itemconfiguration"
    );
    assert_eq!(apis.requests().len(), 1);
    assert_eq!(
        thumbnails.requests()[0].target,
        "/transport-fixtures/thumbnails/2"
    );
    assert_eq!(
        items.requests()[0].target,
        "/transport-fixtures/collectibles/3"
    );
    assert_eq!(items.requests()[0].header("x-api-key"), Some(KEY));
}

// --------------------------------------------------------------- responses

#[test]
fn empty_and_text_responses_are_handled() {
    let server = TestServer::start(vec![
        Reply::empty(204),
        Reply::empty(200),
        Reply::text(200, "42").header("Content-Type", "text/plain; charset=utf-8"),
        Reply::text(200, "[1,2]").header("Content-Type", "text/json"),
    ]);
    let client = client(&server);
    assert_eq!(
        client.execute(fixtures::delete_item("1")).unwrap(),
        Value::Null
    );
    assert_eq!(
        client.execute(fixtures::delete_item("2")).unwrap(),
        Value::Null
    );
    assert_eq!(
        client.execute(fixtures::get_item("3")).unwrap(),
        json!("42")
    );
    assert_eq!(
        client.execute(fixtures::get_item("4")).unwrap(),
        json!([1, 2])
    );
    assert_eq!(server.requests()[0].method, "DELETE");
}

#[test]
fn invalid_json_on_success_is_an_error_without_the_body() {
    let server = TestServer::start(vec![Reply::json(200, &format!("<html>{KEY}</html>"))]);
    let error = client(&server)
        .execute(fixtures::get_item("1"))
        .unwrap_err();
    assert!(
        matches!(error, Error::InvalidResponse { status: 200, .. }),
        "{error:?}"
    );
    assert_no_secret(&both_formats(&error));
}

#[test]
fn oversized_responses_are_refused() {
    let server = TestServer::start(vec![Reply::json(200, &format!("\"{}\"", "x".repeat(5000)))]);
    let client = client_with(policy(&server).max_response_bytes(1000));
    let error = client.execute(fixtures::get_item("1")).unwrap_err();
    assert!(matches!(error, Error::InvalidResponse { .. }), "{error:?}");
    assert!(
        error.to_string().contains("larger than 1000 bytes"),
        "{error}"
    );
}

#[test]
fn server_json_errors_keep_the_status_and_message() {
    let server = TestServer::start(vec![
        Reply::json(
            400,
            r#"{"code":"INVALID_ARGUMENT","message":"Invalid asset id."}"#,
        ),
        Reply::json(
            403,
            r#"{"errors":[{"code":7,"message":"Insufficient scope"}]}"#,
        ),
        Reply::json(
            404,
            r#"{"error":"NOT_FOUND","message":"Entry not found.","errorDetails":[]}"#,
        ),
        Reply::text(500, "upstream exploded"),
        Reply::empty(401),
    ]);
    let client = client(&server);
    let mut seen = Vec::new();
    for id in ["1", "2", "3", "4", "5"] {
        let error = client.execute(fixtures::delete_item(id)).unwrap_err();
        let Error::Status {
            status,
            message,
            operation,
            ..
        } = &error
        else {
            panic!("expected a status error, got {error:?}");
        };
        assert_eq!(operation, "DELETE /transport-fixtures/items/{item_id}");
        assert_eq!(error.status(), Some(*status));
        seen.push((*status, message.clone(), error.to_string()));
    }
    assert_eq!(seen[0].0, 400);
    assert_eq!(seen[0].1, "INVALID_ARGUMENT: Invalid asset id.");
    assert_eq!(
        seen[0].2,
        "DELETE /transport-fixtures/items/{item_id}: HTTP 400: INVALID_ARGUMENT: Invalid asset id."
    );
    assert_eq!(
        (seen[1].0, seen[1].1.as_str()),
        (403, "7: Insufficient scope")
    );
    assert_eq!(
        (seen[2].0, seen[2].1.as_str()),
        (404, "NOT_FOUND: Entry not found.")
    );
    assert_eq!((seen[3].0, seen[3].1.as_str()), (500, "upstream exploded"));
    assert_eq!((seen[4].0, seen[4].1.as_str()), (401, "Unauthorized"));
    assert_eq!(server.requests().len(), 5, "no error here is retried");
}

// --------------------------------------------------------------- redirects

#[test]
fn redirects_are_not_followed_and_the_key_stays_home() {
    let other = TestServer::start(vec![Reply::json(200, r#"{"stolen":true}"#)]);
    let location = format!("http://{}/steal?sig=signed-value-123", other.addr());
    for status in [301u16, 302, 303, 307, 308] {
        let server = TestServer::start(vec![
            Reply::empty(status).header("Location", &location),
            Reply::json(200, r#"{"followed":true}"#),
        ]);
        let error = client(&server)
            .execute(fixtures::get_item("1"))
            .unwrap_err();
        let Error::Redirect {
            status: got,
            location: target,
            ..
        } = &error
        else {
            panic!("{status}: expected a redirect error, got {error:?}");
        };
        assert_eq!(*got, status);
        assert_eq!(
            target.as_deref(),
            Some(format!("http://{}/steal", other.addr()).as_str()),
            "the target is reported without its query"
        );
        let text = both_formats(&error);
        assert!(!text.contains("signed-value-123"), "{text}");
        assert_no_secret(&text);
        assert_eq!(
            server.requests().len(),
            1,
            "{status}: not retried, not followed"
        );
    }
    assert_eq!(
        other.connections(),
        0,
        "the redirect target was never contacted"
    );
}

#[test]
fn a_redirect_without_a_location_is_still_an_error() {
    let server = TestServer::start(vec![Reply::empty(302)]);
    let error = client(&server)
        .execute(fixtures::create_item(&json!({})))
        .unwrap_err();
    assert!(
        matches!(
            error,
            Error::Redirect {
                status: 302,
                location: None,
                ..
            }
        ),
        "{error:?}"
    );
}

// ----------------------------------------------------------------- retries

fn rate_limited() -> Reply {
    Reply::json(
        429,
        r#"{"code":"RESOURCE_EXHAUSTED","message":"Too many requests."}"#,
    )
    .header("Retry-After", "0")
}

#[test]
fn rate_limited_reads_are_retried_until_they_succeed() {
    let server = TestServer::start(vec![
        rate_limited(),
        Reply::empty(503),
        Reply::json(200, r#"{"ok":true}"#),
    ]);
    let client = client_with(policy(&server).max_retries(3));
    assert_eq!(
        client.execute(fixtures::get_item("1")).unwrap(),
        json!({"ok": true})
    );
    let seen = server.requests();
    assert_eq!(seen.len(), 3);
    assert!(seen.iter().all(|r| r.header("x-api-key") == Some(KEY)));
}

#[test]
fn rate_limit_retries_are_bounded() {
    let server = TestServer::start((0..20).map(|_| rate_limited()).collect());
    let client = client_with(policy(&server).max_retries(2));
    let error = client.execute(fixtures::get_item("1")).unwrap_err();
    assert!(
        matches!(
            error,
            Error::Status {
                status: 429,
                attempts: 3,
                ..
            }
        ),
        "{error:?}"
    );
    assert!(error
        .to_string()
        .contains("RESOURCE_EXHAUSTED: Too many requests."));
    assert!(error.to_string().contains("after 3 attempts"), "{error}");
    assert_eq!(server.requests().len(), 3, "one attempt plus two retries");

    let server = TestServer::start((0..5).map(|_| rate_limited()).collect());
    let client = client_with(policy(&server).max_retries(0));
    assert!(client.execute(fixtures::get_item("1")).is_err());
    assert_eq!(
        server.requests().len(),
        1,
        "max_retries(0) means one attempt"
    );
}

#[test]
fn writes_are_not_retried_unless_documented_idempotent() {
    let server = TestServer::start((0..4).map(|_| rate_limited()).collect());
    let client = client_with(policy(&server).max_retries(3));
    let body = json!({"n": 1});

    for request in [
        fixtures::create_item(&body),
        fixtures::update_item("1", "n", &body),
        fixtures::delete_item("1"),
    ] {
        let error = client.execute(request).unwrap_err();
        assert!(
            matches!(
                error,
                Error::Status {
                    status: 429,
                    attempts: 1,
                    ..
                }
            ),
            "{error:?}"
        );
    }
    assert_eq!(
        server.requests().len(),
        3,
        "each write was sent exactly once"
    );

    let server = TestServer::start(vec![rate_limited(), Reply::json(200, r#"{"n":1}"#)]);
    let client = client_with(policy(&server).max_retries(3));
    assert_eq!(
        client.execute(fixtures::replace_item("1", &body)).unwrap(),
        body
    );
    let seen = server.requests();
    assert_eq!(seen.len(), 2, "the idempotent PUT was retried");
    assert_eq!(seen[1].method, "PUT");
    assert_eq!(
        serde_json::from_slice::<Value>(&seen[1].body).unwrap(),
        body
    );
}

#[test]
fn client_errors_are_never_retried() {
    for status in [400u16, 401, 403, 404, 409, 500, 501] {
        let server = TestServer::start(vec![Reply::empty(status), Reply::json(200, "{}")]);
        let error = client(&server)
            .execute(fixtures::get_item("1"))
            .unwrap_err();
        assert_eq!(error.status(), Some(status));
        assert_eq!(server.requests().len(), 1, "{status}");
    }
}

#[test]
fn retry_waits_are_capped_by_the_policy() {
    let server = TestServer::start(vec![
        Reply::empty(429).header("Retry-After", "15"),
        Reply::empty(429).header("x-ratelimit-reset", "15"),
        Reply::json(200, "{}"),
    ]);
    let client = client_with(
        policy(&server)
            .max_retries(2)
            .max_retry_delay(Duration::from_millis(20)),
    );
    let started = Instant::now();
    client.execute(fixtures::get_item("1")).unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(3),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(server.requests().len(), 3);
}

#[test]
fn dropped_connections_are_retried_for_reads_only() {
    let server = TestServer::start(vec![Reply::close(), Reply::json(200, r#"{"ok":true}"#)]);
    assert_eq!(
        client(&server).execute(fixtures::get_item("1")).unwrap(),
        json!({"ok": true})
    );
    assert_eq!(server.requests().len(), 2);

    let server = TestServer::start(vec![Reply::close(), Reply::json(200, "{}")]);
    let error = client(&server)
        .execute(fixtures::create_item(&json!({})))
        .unwrap_err();
    assert!(
        matches!(error, Error::Transport { attempts: 1, .. }),
        "{error:?}"
    );
    assert_eq!(server.requests().len(), 1, "a write is not re-sent");
}

#[test]
fn timeouts_and_refused_connections_are_bounded_and_redacted() {
    let server = TestServer::start(vec![
        Reply::hang(Duration::from_millis(600)),
        Reply::hang(Duration::from_millis(600)),
    ]);
    let client = client_with(
        policy(&server)
            .timeout(Duration::from_millis(100))
            .max_retries(1),
    );
    let error = client
        .execute(fixtures::list_items(None, Some("signed-token-value"), &[]))
        .unwrap_err();
    assert!(
        matches!(
            error,
            Error::Transport {
                kind: TransportKind::Timeout,
                attempts: 2,
                ..
            }
        ),
        "{error:?}"
    );
    let text = both_formats(&error);
    assert!(!text.contains("signed-token-value"), "{text}");
    assert_no_secret(&text);

    // A port with nothing listening.
    let free = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = free.local_addr().unwrap();
    drop(free);
    let client = client_with(
        ClientPolicy::loopback(addr)
            .unwrap()
            .retry_delay(Duration::from_millis(1))
            .max_retries(1),
    );
    let error = client
        .execute(fixtures::list_items(None, Some("signed-token-value"), &[]))
        .unwrap_err();
    assert!(
        matches!(
            error,
            Error::Transport {
                kind: TransportKind::Connect,
                attempts: 2,
                ..
            }
        ),
        "{error:?}"
    );
    let text = both_formats(&error);
    assert!(!text.contains("signed-token-value"), "{text}");
    assert!(!text.contains("pageToken"), "{text}");
    assert_no_secret(&text);
}

// ----------------------------------------------------------------- polling

fn pending() -> Reply {
    Reply::json(
        200,
        r#"{"path":"operations/job-1","operationId":"job-1","done":false}"#,
    )
}

#[test]
fn operations_are_polled_until_done() {
    let server = TestServer::start(vec![
        pending(),
        pending(),
        pending(),
        Reply::json(
            200,
            r#"{"operationId":"job-1","done":true,"response":{"assetId":"55"}}"#,
        ),
    ]);
    let done = client(&server)
        .execute(fixtures::start_job(&json!({"n": 1})))
        .unwrap();
    assert_eq!(done["response"]["assetId"], "55");

    let seen = server.requests();
    assert_eq!(seen.len(), 4);
    assert_eq!(
        (seen[0].method.as_str(), seen[0].target.as_str()),
        ("POST", "/transport-fixtures/jobs")
    );
    for check in &seen[1..] {
        assert_eq!(check.method, "GET");
        assert_eq!(check.target, "/transport-fixtures/jobs/job-1");
        assert_eq!(check.header("x-api-key"), Some(KEY));
        assert!(check.body.is_empty());
    }
}

#[test]
fn an_operation_that_is_done_at_once_is_not_polled() {
    let server = TestServer::start(vec![Reply::json(200, r#"{"done":true,"response":{}}"#)]);
    client(&server)
        .execute(fixtures::start_job(&json!({})))
        .unwrap();
    assert_eq!(server.requests().len(), 1);
}

#[test]
fn operation_polling_is_bounded() {
    let server = TestServer::start((0..50).map(|_| pending()).collect());
    let client = client_with(policy(&server).max_poll_attempts(3));
    let error = client.execute(fixtures::start_job(&json!({}))).unwrap_err();
    assert!(
        matches!(error, Error::OperationTimedOut { attempts: 3, .. }),
        "{error:?}"
    );
    assert_eq!(
        error.to_string(),
        "POST /transport-fixtures/jobs: operation still running after 3 status check(s)"
    );
    assert_eq!(
        server.requests().len(),
        4,
        "one start plus exactly three checks"
    );

    let server = TestServer::start((0..5).map(|_| pending()).collect());
    let client = client_with(policy(&server).max_poll_attempts(0));
    let error = client.execute(fixtures::start_job(&json!({}))).unwrap_err();
    assert!(
        matches!(error, Error::OperationTimedOut { attempts: 0, .. }),
        "{error:?}"
    );
    assert_eq!(
        server.requests().len(),
        1,
        "max_poll_attempts(0) never polls"
    );
}

#[test]
fn the_start_request_of_an_operation_is_sent_once() {
    let server = TestServer::start(vec![rate_limited(), pending()]);
    let client = client_with(policy(&server).max_retries(3));
    let error = client.execute(fixtures::start_job(&json!({}))).unwrap_err();
    assert!(
        matches!(
            error,
            Error::Status {
                status: 429,
                attempts: 1,
                ..
            }
        ),
        "{error:?}"
    );
    assert_eq!(server.requests().len(), 1);
}

#[test]
fn status_checks_are_retried_within_the_retry_limit() {
    let server = TestServer::start(vec![
        pending(),
        rate_limited(),
        Reply::json(200, r#"{"done":true,"response":{"ok":true}}"#),
    ]);
    let client = client_with(policy(&server).max_retries(1).max_poll_attempts(1));
    let done = client.execute(fixtures::start_job(&json!({}))).unwrap();
    assert_eq!(done["response"]["ok"], true);
    assert_eq!(server.requests().len(), 3);
}

#[test]
fn failed_and_malformed_operations_are_errors() {
    let server = TestServer::start(vec![
        pending(),
        Reply::json(
            200,
            &format!(
                r#"{{"done":true,"error":{{"code":"INVALID_ARGUMENT","message":"bad file {KEY} at https://cdn.test/f?sig=abc"}}}}"#
            ),
        ),
    ]);
    let error = client(&server)
        .execute(fixtures::start_job(&json!({})))
        .unwrap_err();
    let Error::OperationFailed { message, .. } = &error else {
        panic!("expected a failed operation, got {error:?}");
    };
    assert_eq!(
        message,
        "INVALID_ARGUMENT: bad file [REDACTED] at https://cdn.test/f?[REDACTED]"
    );

    let server = TestServer::start(vec![Reply::json(200, r#"{"done":false}"#)]);
    let error = client(&server)
        .execute(fixtures::start_job(&json!({})))
        .unwrap_err();
    assert!(matches!(error, Error::InvalidResponse { .. }), "{error:?}");
    assert!(error.to_string().contains("no operationId"), "{error}");
    assert_eq!(server.requests().len(), 1);
}

// --------------------------------------------------------------- multipart

#[test]
fn multipart_sends_exact_names_and_bytes() {
    let server = TestServer::start(vec![
        Reply::json(200, r#"{"operationId":"up-1"}"#),
        Reply::json(200, r#"{"operationId":"up-2"}"#),
    ]);
    let client = client(&server);
    let metadata = json!({"assetType": "Decal", "displayName": "Tëst \"quoted\""});

    // Every byte value, plus CRLF runs and boundary-like dashes.
    let mut data: Vec<u8> = (0u8..=255).collect();
    data.extend_from_slice(b"\r\n--\r\n\r\n--boundary--\r\n");
    data.extend((0u8..=255).rev());

    client
        .execute(
            fixtures::upload_bytes(&metadata, "pic ture.png", "image/png", data.clone()).unwrap(),
        )
        .unwrap();

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("model file.fbx");
    fs::write(&path, &data).unwrap();
    client
        .execute(fixtures::upload_file(&metadata, &path, "model/fbx").unwrap())
        .unwrap();

    let seen = server.requests();
    for (request, file_name, content_type) in [
        (&seen[0], "pic ture.png", "image/png"),
        (&seen[1], "model file.fbx", "model/fbx"),
    ] {
        assert_eq!(request.method, "POST");
        assert_eq!(request.target, "/transport-fixtures/uploads");
        assert_eq!(request.header("x-api-key"), Some(KEY));
        let header = request.header("content-type").unwrap();
        assert!(
            header.starts_with("multipart/form-data; boundary="),
            "{header}"
        );
        assert_eq!(
            request.header("content-length"),
            Some(request.body.len().to_string().as_str())
        );

        let parts = parse_multipart(header, &request.body);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].name, "request");
        assert_eq!(parts[0].file_name, None);
        assert_eq!(
            serde_json::from_slice::<Value>(&parts[0].data).unwrap(),
            metadata
        );
        assert_eq!(parts[1].name, "fileContent");
        assert_eq!(parts[1].file_name.as_deref(), Some(file_name));
        assert_eq!(parts[1].content_type.as_deref(), Some(content_type));
        assert_eq!(parts[1].data, data, "file bytes arrive unchanged");
    }
}

#[test]
fn multipart_uploads_are_not_retried() {
    let server = TestServer::start(vec![rate_limited(), Reply::json(200, "{}")]);
    let client = client_with(policy(&server).max_retries(3));
    let error = client
        .execute(fixtures::upload_bytes(&json!({}), "a.png", "image/png", vec![1, 2, 3]).unwrap())
        .unwrap_err();
    assert_eq!(error.status(), Some(429));
    assert_eq!(server.requests().len(), 1);
}

#[test]
fn unreadable_attachments_and_bad_content_types_fail_before_sending() {
    let server = TestServer::start(vec![]);
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing.png");
    let error = fixtures::upload_file(&json!({}), &missing, "image/png").unwrap_err();
    assert!(
        matches!(&error, Error::File { path, kind } if path == &missing && *kind == std::io::ErrorKind::NotFound),
        "{error:?}"
    );

    let error = client(&server)
        .execute(fixtures::upload_bytes(&json!({}), "a.png", "not a mime", vec![1]).unwrap())
        .unwrap_err();
    assert!(matches!(error, Error::InvalidRequest { .. }), "{error:?}");
    assert_eq!(server.connections(), 0);
}

// --------------------------------------------------------------- redaction

#[test]
fn errors_never_contain_the_key_or_signed_queries() {
    let echo = format!(
        r#"{{"code":"PERMISSION_DENIED","message":"key {KEY} rejected; see https://files.test/dl/abc?X-Amz-Signature=deadbeef&Expires=1 for details"}}"#
    );
    let server = TestServer::start(vec![
        Reply::json(403, &echo),
        Reply::text(500, &format!("raw {KEY} https://files.test/x?token=abc123")),
    ]);
    let client = client(&server);

    for _ in 0..2 {
        let error = client
            .execute(fixtures::create_item(
                &json!({"secretField": "body-secret-value"}),
            ))
            .unwrap_err();
        let text = both_formats(&error);
        assert_no_secret(&text);
        assert!(!text.contains("deadbeef"), "{text}");
        assert!(!text.contains("abc123"), "{text}");
        assert!(
            !text.contains("body-secret-value"),
            "request body leaked: {text}"
        );
        assert!(text.contains("[REDACTED]"), "{text}");
        assert!(
            text.contains("https://files.test/"),
            "the URL path is kept: {text}"
        );
    }
    assert_eq!(
        client
            .execute(fixtures::get_item("1"))
            .map_err(|e| e.to_string())
            .unwrap_err(),
        "GET /transport-fixtures/items/{item_id}: HTTP 500: test server script exhausted"
    );
}

#[test]
fn error_messages_are_length_capped() {
    let server = TestServer::start(vec![Reply::text(400, &"A".repeat(50_000))]);
    let error = client(&server)
        .execute(fixtures::get_item("1"))
        .unwrap_err();
    assert!(
        error.to_string().len() < 2_200,
        "{}",
        error.to_string().len()
    );
}

#[test]
fn errors_name_the_operation_template_not_the_filled_in_url() {
    let server = TestServer::start(vec![Reply::empty(404)]);
    let error = client(&server)
        .execute(fixtures::get_item("private-identifier-777"))
        .unwrap_err();
    let text = both_formats(&error);
    assert!(
        text.contains("GET /transport-fixtures/items/{item_id}"),
        "{text}"
    );
    assert!(!text.contains("private-identifier-777"), "{text}");
}

#[test]
fn a_key_echoed_in_a_success_body_is_masked() {
    let body = format!(
        r#"{{"apiKey":"{KEY}","note":"prefix {KEY} suffix","url":"https://files.test/dl?sig=keep-me","list":["{KEY}"]}}"#
    );
    let server = TestServer::start(vec![
        Reply::json(200, &body),
        Reply::text(200, &format!("plain {KEY}")).header("Content-Type", "text/plain"),
    ]);
    let client = client(&server);
    let value = client.execute(fixtures::get_item("1")).unwrap();
    assert_no_secret(&value.to_string());
    assert_eq!(value["apiKey"], "[REDACTED]");
    assert_eq!(value["note"], "prefix [REDACTED] suffix");
    assert_eq!(
        value["url"], "https://files.test/dl?sig=keep-me",
        "signed URLs in a successful response are the payload and are returned intact"
    );
    assert_eq!(
        client.execute(fixtures::get_item("2")).unwrap(),
        json!("plain [REDACTED]")
    );
}

// ------------------------------------------------------------- time limits

#[test]
fn durations_have_hard_ceilings() {
    let new = |policy| Client::new(Credentials::api_key(SecretString::new(KEY)), policy);
    let ok = ClientPolicy::default;
    let rejected = [
        ("timeout", ok().timeout(Duration::from_secs(601))),
        ("retry_delay", ok().retry_delay(Duration::from_secs(301))),
        (
            "max_retry_delay",
            ok().max_retry_delay(Duration::from_secs(301)),
        ),
        ("poll_delay", ok().poll_delay(Duration::from_secs(301))),
        (
            "max_total_time",
            ok().max_total_time(Duration::from_secs(3601)),
        ),
        ("max_total_time", ok().max_total_time(Duration::ZERO)),
    ];
    for (name, policy) in rejected {
        match new(policy) {
            Err(Error::Config(reason)) => assert!(reason.contains(name), "{name}: {reason}"),
            other => panic!("{name}: expected a configuration error, got {other:?}"),
        }
    }
    let at_the_limits = ok()
        .timeout(Duration::from_secs(600))
        .retry_delay(Duration::from_secs(300))
        .max_retry_delay(Duration::from_secs(300))
        .poll_delay(Duration::from_secs(300))
        .max_total_time(Duration::from_secs(3600));
    assert!(new(at_the_limits).is_ok());
}

#[test]
fn a_slowly_dripped_body_cannot_outlast_the_attempt_timeout() {
    // 400 bytes at one per 50 ms would take 20 s. Every single read succeeds
    // well inside the timeout, so only a clock over the whole attempt ends it.
    let body = format!("\"{}\"", "x".repeat(398));
    let server = TestServer::start(vec![Reply::drip(200, &body, Duration::from_millis(50))]);
    let client = client_with(
        policy(&server)
            .timeout(Duration::from_millis(400))
            .max_retries(0),
    );
    let started = Instant::now();
    let error = client.execute(fixtures::get_item("1")).unwrap_err();
    let elapsed = started.elapsed();
    assert!(
        matches!(
            error,
            Error::Transport {
                kind: TransportKind::Timeout,
                attempts: 1,
                ..
            }
        ),
        "{error:?}"
    );
    assert!(elapsed >= Duration::from_millis(350), "{elapsed:?}");
    assert!(elapsed < Duration::from_secs(3), "{elapsed:?}");
}

#[test]
fn a_body_that_stalls_part_way_ends_at_the_attempt_timeout() {
    // The head arrives at once, then nothing for 3 s.
    let server = TestServer::start(vec![Reply::drip(200, "{}", Duration::from_secs(3))]);
    let client = client_with(
        policy(&server)
            .timeout(Duration::from_millis(300))
            .max_retries(0),
    );
    let started = Instant::now();
    let error = client.execute(fixtures::get_item("1")).unwrap_err();
    let elapsed = started.elapsed();
    assert!(
        matches!(
            error,
            Error::Transport {
                kind: TransportKind::Timeout,
                ..
            }
        ),
        "{error:?}"
    );
    assert!(elapsed < Duration::from_millis(1500), "{elapsed:?}");
}

#[test]
fn a_dripped_body_that_fits_the_timeout_is_read_whole() {
    let server = TestServer::start(vec![Reply::drip(
        200,
        r#"{"ok":true}"#,
        Duration::from_millis(5),
    )]);
    assert_eq!(
        client(&server).execute(fixtures::get_item("1")).unwrap(),
        json!({"ok": true})
    );
}

#[test]
fn polling_stops_at_the_overall_deadline() {
    // The count limit alone would allow 1000 checks.
    let server = TestServer::start((0..1000).map(|_| pending()).collect());
    let client = client_with(
        policy(&server)
            .max_poll_attempts(1000)
            .poll_delay(Duration::from_millis(25))
            .max_total_time(Duration::from_millis(400)),
    );
    let started = Instant::now();
    let error = client.execute(fixtures::start_job(&json!({}))).unwrap_err();
    let elapsed = started.elapsed();
    let Error::DeadlineExceeded {
        checks,
        limit,
        operation,
    } = &error
    else {
        panic!("expected a deadline error, got {error:?}");
    };
    assert_eq!(*limit, Duration::from_millis(400));
    assert_eq!(operation, "POST /transport-fixtures/jobs");
    assert!((1..=16).contains(checks), "{checks} checks");
    assert_eq!(server.requests().len() as u32, checks + 1);
    assert!(elapsed < Duration::from_secs(3), "{elapsed:?}");
    assert!(error.to_string().contains("time limit"), "{error}");
}

#[test]
fn retries_stop_at_the_overall_deadline() {
    // Backoff 100, 200, 400 ms: the third wait would pass the deadline, so
    // the last real failure is returned instead of sleeping.
    let server = TestServer::start((0..10).map(|_| Reply::empty(429)).collect());
    let client = client_with(
        policy(&server)
            .max_retries(10)
            .retry_delay(Duration::from_millis(100))
            .max_total_time(Duration::from_millis(500)),
    );
    let started = Instant::now();
    let error = client.execute(fixtures::get_item("1")).unwrap_err();
    let elapsed = started.elapsed();
    let Error::Status {
        status: 429,
        attempts,
        ..
    } = error
    else {
        panic!("expected the 429, got {error:?}");
    };
    assert!((2..=3).contains(&attempts), "{attempts} attempts");
    assert_eq!(server.requests().len() as u32, attempts);
    assert!(elapsed < Duration::from_secs(2), "{elapsed:?}");
}

#[test]
fn the_overall_deadline_cuts_a_stalled_attempt_short() {
    // The attempt timeout alone would allow 5 s, three times over.
    let hang = || Reply::hang(Duration::from_secs(2));
    let server = TestServer::start(vec![hang(), hang(), hang(), hang()]);
    let client = client_with(
        policy(&server)
            .timeout(Duration::from_secs(5))
            .max_retries(3)
            .max_total_time(Duration::from_millis(300)),
    );
    let started = Instant::now();
    let error = client.execute(fixtures::get_item("1")).unwrap_err();
    let elapsed = started.elapsed();
    assert!(
        matches!(
            error,
            Error::Transport {
                kind: TransportKind::Timeout,
                attempts: 1,
                ..
            }
        ),
        "{error:?}"
    );
    assert!(elapsed < Duration::from_millis(1500), "{elapsed:?}");
    assert_eq!(
        server.requests().len(),
        1,
        "no retry is started after the deadline"
    );
}

// ------------------------------------------------------------------- async

fn key() -> Credentials {
    Credentials::api_key(SecretString::new(KEY))
}

/// Runs `work` and counts how many 10 ms ticks the runtime managed meanwhile.
async fn ticks_during<T>(work: impl std::future::Future<Output = T>) -> (T, u32) {
    tokio::pin!(work);
    let mut ticks = 0u32;
    let mut interval = tokio::time::interval(Duration::from_millis(10));
    loop {
        tokio::select! {
            output = &mut work => return (output, ticks),
            _ = interval.tick() => ticks += 1,
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn async_client_does_not_block_the_only_runtime_thread() {
    // Three status checks 80 ms apart: at least 240 ms inside the transport.
    let server = TestServer::start(vec![
        pending(),
        pending(),
        pending(),
        Reply::json(200, r#"{"done":true,"response":{"ok":true}}"#),
    ]);
    let policy = policy(&server).poll_delay(Duration::from_millis(80));
    let client = AsyncClient::new(key(), policy).await.unwrap();

    let (result, ticks) = ticks_during(client.execute(fixtures::start_job(&json!({})))).await;
    assert_eq!(result.unwrap()["response"]["ok"], true);
    // A blocked single-threaded runtime could not have ticked at all.
    assert!(ticks >= 10, "the runtime only ticked {ticks} times");
    assert_eq!(server.requests().len(), 4);
    assert_eq!(server.requests()[0].header("x-api-key"), Some(KEY));
    drop(client);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn async_client_works_on_a_multi_thread_runtime_and_when_cloned() {
    let server = TestServer::start((0..8).map(|_| Reply::json(200, r#"{"ok":true}"#)).collect());
    let client = AsyncClient::new(key(), policy(&server)).await.unwrap();
    assert!(!client.policy().is_production());

    let calls: Vec<_> = (0..8)
        .map(|n| {
            let client = client.clone();
            tokio::spawn(async move { client.execute(fixtures::get_item(&n.to_string())).await })
        })
        .collect();
    for call in calls {
        assert_eq!(call.await.unwrap().unwrap(), json!({"ok": true}));
    }
    assert_eq!(server.requests().len(), 8);
}

#[tokio::test(flavor = "current_thread")]
async fn async_client_run_gives_a_sequence_of_calls_one_blocking_task() {
    let server = TestServer::start(vec![
        Reply::json(200, r#"{"items":[1],"nextPageToken":"p2"}"#),
        Reply::json(200, r#"{"items":[2]}"#),
    ]);
    let client = AsyncClient::new(key(), policy(&server)).await.unwrap();
    let items = client
        .run(|client| {
            let mut items = Vec::new();
            let mut token: Option<String> = None;
            loop {
                let page = client.execute(fixtures::list_items(None, token.as_deref(), &[]))?;
                items.extend(page["items"].as_array().cloned().unwrap_or_default());
                match page["nextPageToken"].as_str() {
                    Some(next) => token = Some(next.to_string()),
                    None => return Ok(items),
                }
            }
        })
        .await
        .unwrap();
    assert_eq!(items, vec![json!(1), json!(2)]);
    assert_eq!(
        server.requests()[1].target,
        "/transport-fixtures/items?pageToken=p2"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn async_client_reports_errors_and_a_panicked_task() {
    let server = TestServer::start(vec![Reply::json(404, r#"{"message":"gone"}"#)]);
    let client = AsyncClient::new(key(), policy(&server)).await.unwrap();
    let error = client.execute(fixtures::get_item("1")).await.unwrap_err();
    assert_eq!(error.status(), Some(404));

    let error = client
        .run(|_| -> Result<(), Error> { panic!("deliberate test panic") })
        .await
        .unwrap_err();
    assert!(matches!(error, Error::TaskFailed { .. }), "{error:?}");

    let bad = AsyncClient::new(key(), ClientPolicy::default().max_retries(99)).await;
    assert!(matches!(bad, Err(Error::Config(_))));
}

#[tokio::test(flavor = "current_thread")]
async fn the_blocking_client_is_refused_on_a_runtime_thread_instead_of_panicking() {
    let server = TestServer::start(vec![Reply::json(200, "{}")]);

    // Building it here would panic inside reqwest in a debug build.
    let built = Client::new(key(), policy(&server));
    let Err(Error::Config(reason)) = built else {
        panic!("expected a configuration error");
    };
    assert!(reason.contains("AsyncClient"), "{reason}");

    // A client built elsewhere is refused when used here.
    let policy = policy(&server);
    let client = std::thread::spawn(move || Client::new(key(), policy).unwrap())
        .join()
        .unwrap();
    let error = client.execute(fixtures::get_item("1")).unwrap_err();
    assert!(matches!(error, Error::Config(_)), "{error:?}");
    assert_eq!(server.connections(), 0, "nothing was sent");

    // The same client works once it is off the runtime thread.
    let value = std::thread::spawn(move || client.execute(fixtures::get_item("1")))
        .join()
        .unwrap();
    assert_eq!(value.unwrap(), json!({}));
}

// ------------------------------------------------------ attachment names

#[test]
fn attachment_names_with_line_breaks_or_quotes_are_refused_before_sending() {
    let server = TestServer::start(vec![]);
    let long = "n".repeat(256);
    let names = [
        "evil.png\r\nContent-Type: text/html\r\n\r\n<script>",
        "a\nb.png",
        "a\rb.png",
        "a\u{0}b.png",
        "a\tb.png",
        "a\"b.png",
        "dir/a.png",
        "dir\\a.png",
        "",
        "  ",
        "..",
        long.as_str(),
    ];
    for name in names {
        let error =
            fixtures::upload_bytes(&json!({}), name, "image/png", vec![1, 2, 3]).unwrap_err();
        assert!(
            matches!(error, Error::InvalidAttachment { .. }),
            "{name:?}: {error:?}"
        );
        let text = both_formats(&error);
        assert!(
            !text.contains("text/html") && !text.contains("<script>"),
            "{text}"
        );
    }
    assert_eq!(server.connections(), 0);
    let _ = client(&server);
}

#[cfg(unix)]
#[test]
fn a_file_whose_name_has_a_line_break_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("evil.png\r\nX-Injected: 1");
    fs::write(&path, [1, 2, 3]).unwrap();
    let error = fixtures::upload_file(&json!({}), &path, "image/png").unwrap_err();
    assert!(
        matches!(error, Error::InvalidAttachment { .. }),
        "{error:?}"
    );
    assert!(!both_formats(&error).contains("X-Injected"));
}
