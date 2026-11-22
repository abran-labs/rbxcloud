//! Checks `openapi/coverage.json` against the checked-in official spec snapshot
//! `openapi/roblox.json`. No network access and no credentials.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

const HTTP_METHODS: [&str; 8] = [
    "get", "post", "put", "patch", "delete", "head", "options", "trace",
];

/// Words that would make a CLI owner a generic passthrough instead of a typed command.
const GENERIC_WORDS: [&str; 20] = [
    "api",
    "request",
    "raw",
    "call",
    "http",
    "fetch",
    "curl",
    "send-request",
    "passthrough",
    "proxy",
    "generic",
    "invoke",
    "exec",
    "cloud",
    "get-request",
    "post",
    "put",
    "patch",
    "head",
    "options",
];

/// `existing`: a legacy client reaches the operation. `implemented`: typed method on the
/// shared transport plus an explicit command. `defective`: a legacy command targets the
/// operation but does not send the documented request. `pending`: planned only.
const STATUSES: [&str; 4] = ["existing", "implemented", "defective", "pending"];

/// Statuses that count as covered.
const COVERED: [&str; 2] = ["existing", "implemented"];

/// Catch-all buckets that are not a typed family.
const GENERIC_FAMILIES: [&str; 10] = [
    "cloud", "api", "apis", "other", "misc", "unknown", "generic", "legacy", "v1", "v2",
];

fn repo_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read(rel: &str) -> Vec<u8> {
    let path = repo_path(rel);
    std::fs::read(&path).unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn spec() -> Value {
    serde_json::from_slice(&read("openapi/roblox.json")).expect("openapi/roblox.json is not JSON")
}

fn manifest() -> Value {
    serde_json::from_slice(&read("openapi/coverage.json"))
        .expect("openapi/coverage.json is not JSON")
}

fn records<'a>(manifest: &'a Value, key: &str) -> &'a Vec<Value> {
    manifest[key]
        .as_array()
        .unwrap_or_else(|| panic!("manifest `{key}` must be an array"))
}

fn text<'a>(record: &'a Value, key: &str) -> &'a str {
    record[key]
        .as_str()
        .unwrap_or_else(|| panic!("`{key}` must be a string in {record}"))
}

/// SHA-256 (FIPS 180-4). Kept local so this test adds no dependency.
// `as_chunks` needs Rust 1.88; the lint that suggests it is unknown to older clippy.
#[allow(unknown_lints, clippy::chunks_exact_to_as_chunks)]
fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let mut message = data.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());

    for block in message.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (slot, word) in w.iter_mut().zip(block.chunks_exact(4)) {
            *slot = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (slot, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(value);
        }
    }

    state.iter().map(|word| format!("{word:08x}")).collect()
}

struct SpecOperation {
    security: Value,
    operation_id: Value,
    tags: Value,
    stability: Value,
    server: String,
}

fn single_server(servers: &Value) -> Option<String> {
    let servers = servers.as_array()?;
    assert_eq!(servers.len(), 1, "expected exactly one server: {servers:?}");
    Some(servers[0]["url"].as_str()?.to_string())
}

/// Every operation in the spec, keyed by `HTTP_METHOD /path`.
fn spec_operations(spec: &Value) -> BTreeMap<String, SpecOperation> {
    let mut out = BTreeMap::new();
    let default_server = single_server(&spec["servers"]).expect("spec has no default server");
    let paths = spec["paths"].as_object().expect("spec has no paths");
    for (path, item) in paths {
        for method in HTTP_METHODS {
            if let Some(operation) = item.get(method) {
                let id = format!("{} {path}", method.to_uppercase());
                let server = single_server(&operation["servers"])
                    .or_else(|| single_server(&item["servers"]))
                    .unwrap_or_else(|| default_server.clone());
                out.insert(
                    id,
                    SpecOperation {
                        security: operation["security"].clone(),
                        operation_id: operation["operationId"].clone(),
                        tags: operation
                            .get("tags")
                            .cloned()
                            .unwrap_or_else(|| Value::Array(vec![])),
                        stability: operation["x-roblox-stability"].clone(),
                        server,
                    },
                );
            }
        }
    }
    out
}

/// The auth names the manifest uses for a spec `security` value.
fn auth_names(security: &Value) -> Vec<&'static str> {
    let Some(requirements) = security.as_array() else {
        return vec!["unspecified"];
    };
    let has = |scheme: &str| requirements.iter().any(|r| r.get(scheme).is_some());
    let mut out = vec![];
    if has("roblox-api-key") {
        out.push("api-key");
    }
    if has("roblox-oauth2") {
        out.push("oauth2");
    }
    if has("roblox-legacy-cookie") {
        out.push("legacy-cookie");
    }
    if requirements
        .iter()
        .any(|r| r.as_object().is_some_and(|schemes| schemes.is_empty()))
    {
        out.push("none");
    }
    out
}

/// API-key eligible: one alternative in `security` is exactly `{"roblox-api-key": []}`.
fn accepts_api_key(security: &Value) -> bool {
    security.as_array().is_some_and(|requirements| {
        requirements.iter().any(|requirement| {
            requirement
                .as_object()
                .is_some_and(|schemes| schemes.len() == 1 && schemes.contains_key("roblox-api-key"))
        })
    })
}

fn mentions_scheme(security: &Value, scheme: &str) -> bool {
    security.as_array().is_some_and(|requirements| {
        requirements
            .iter()
            .any(|requirement| requirement.get(scheme).is_some())
    })
}

fn is_kebab(word: &str) -> bool {
    !word.is_empty()
        && !word.starts_with('-')
        && !word.ends_with('-')
        && word
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// The `<group> <command>` tokens of a `rbxcloud <group> <command>` owner.
fn cli_tokens(cli: &str) -> Vec<&str> {
    let tokens: Vec<&str> = cli.split(' ').collect();
    assert!(
        tokens.len() == 3 && tokens[0] == "rbxcloud",
        "CLI owner must be `rbxcloud <group> <command>`, got `{cli}`"
    );
    tokens[1..].to_vec()
}

fn help_succeeds(tokens: &[&str]) -> bool {
    Command::new(env!("CARGO_BIN_EXE_rbxcloud"))
        .args(tokens)
        .arg("--help")
        .env_remove("RBXCLOUD_API_KEY")
        .output()
        .expect("failed to run rbxcloud")
        .status
        .success()
}

#[test]
fn sha256_matches_known_vectors() {
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
    // 64 bytes: padding spills into a second block.
    assert_eq!(
        sha256_hex(&[b'a'; 64]),
        "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb"
    );
}

#[test]
fn spec_sha256_matches_snapshot() {
    let manifest = manifest();
    assert_eq!(manifest["schema_version"], 1);
    assert_eq!(manifest["spec"]["path"], "openapi/roblox.json");
    assert!(text(&manifest["spec"], "source_url").starts_with("https://create.roblox.com/"));
    assert_eq!(
        text(&manifest, "spec_sha256"),
        sha256_hex(&read("openapi/roblox.json")),
        "openapi/coverage.json was not generated from this openapi/roblox.json"
    );
}

#[test]
fn operation_ids_are_unique_and_stable() {
    let manifest = manifest();
    let mut seen = BTreeSet::new();
    for key in ["operations", "excluded"] {
        for record in records(&manifest, key) {
            let id = text(record, "id");
            assert_eq!(
                id,
                format!("{} {}", text(record, "method"), text(record, "path")),
                "id must be `HTTP_METHOD /path`"
            );
            assert!(
                HTTP_METHODS.contains(&text(record, "method").to_lowercase().as_str())
                    && text(record, "method") == text(record, "method").to_uppercase(),
                "bad method in {id}"
            );
            assert!(text(record, "path").starts_with('/'), "bad path in {id}");
            assert!(seen.insert(id.to_string()), "duplicate operation id {id}");
        }
    }
}

#[test]
fn manifest_accounts_for_every_spec_operation() {
    let spec = spec();
    let manifest = manifest();
    let spec_ops = spec_operations(&spec);

    let expected_required: BTreeSet<&str> = spec_ops
        .iter()
        .filter(|(_, operation)| accepts_api_key(&operation.security))
        .map(|(id, _)| id.as_str())
        .collect();
    let expected_excluded: BTreeSet<&str> = spec_ops
        .iter()
        .filter(|(_, operation)| !accepts_api_key(&operation.security))
        .map(|(id, _)| id.as_str())
        .collect();
    let required: BTreeSet<&str> = records(&manifest, "operations")
        .iter()
        .map(|record| text(record, "id"))
        .collect();
    let excluded: BTreeSet<&str> = records(&manifest, "excluded")
        .iter()
        .map(|record| text(record, "id"))
        .collect();

    assert!(
        !required.is_empty(),
        "no API-key operations in the manifest"
    );
    assert_eq!(
        required, expected_required,
        "`operations` must be exactly the spec operations that accept an API key"
    );
    assert_eq!(
        excluded, expected_excluded,
        "`excluded` must be exactly the spec operations that do not accept an API key"
    );

    for id in [
        "GET /cloud/v2/universes/{universe_id}/data-stores",
        "POST /assets/v1/assets",
    ] {
        assert!(required.contains(id), "missing required operation {id}");
    }
}

#[test]
fn api_key_records_have_typed_owners() {
    let manifest = manifest();
    let operations = records(&manifest, "operations");
    assert!(!operations.is_empty());

    for record in operations {
        let id = text(record, "id");

        let auth: Vec<&str> = record["auth"]
            .as_array()
            .unwrap_or_else(|| panic!("`auth` must be an array in {id}"))
            .iter()
            .map(|scheme| scheme.as_str().expect("auth scheme must be a string"))
            .collect();
        assert!(auth.contains(&"api-key"), "{id} is not API-key auth");

        let family = text(record, "family");
        assert!(
            !family.is_empty()
                && !family.starts_with('_')
                && !family.ends_with('_')
                && family
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            "{id}: family `{family}` is not a snake_case module name"
        );
        assert!(
            !GENERIC_FAMILIES.contains(&family),
            "{id}: family `{family}` is a catch-all bucket"
        );

        let cli = text(record, "cli");
        let mut owners = vec![cli];
        if let Some(aliases) = record.get("cli_aliases") {
            assert_ne!(
                text(record, "status"),
                "pending",
                "{id}: aliases need a real owner"
            );
            owners.extend(
                aliases
                    .as_array()
                    .expect("`cli_aliases` must be an array")
                    .iter()
                    .map(|alias| alias.as_str().expect("alias must be a string")),
            );
        }
        for owner in owners {
            for token in cli_tokens(owner) {
                assert!(
                    is_kebab(token),
                    "{id}: `{token}` in `{owner}` is not kebab-case"
                );
                assert!(
                    !GENERIC_WORDS.contains(&token),
                    "{id}: `{owner}` is a generic request command"
                );
            }
        }

        let status = text(record, "status");
        assert!(
            STATUSES.contains(&status),
            "{id}: unexpected status `{status}`"
        );
        if let Some(note) = record.get("note") {
            assert!(
                note.as_str().is_some_and(|note| !note.trim().is_empty()),
                "{id}: `note` must be a non-empty string"
            );
        }
        if status == "defective" {
            assert!(
                record.get("note").is_some(),
                "{id}: a defective record must say what is wrong in `note`"
            );
        }
    }
}

#[test]
fn pending_owners_are_distinct() {
    let manifest = manifest();
    let mut pending = BTreeSet::new();
    let mut existing = BTreeSet::new();
    for record in records(&manifest, "operations") {
        let cli = text(record, "cli");
        if text(record, "status") == "pending" {
            assert!(
                pending.insert(cli),
                "`{cli}` owns more than one pending operation"
            );
        } else {
            existing.insert(cli);
            if let Some(aliases) = record.get("cli_aliases") {
                existing.extend(aliases.as_array().unwrap().iter().filter_map(Value::as_str));
            }
        }
    }
    for entry in records(&manifest, "outside_spec") {
        existing.insert(text(entry, "cli"));
    }
    let clash: Vec<_> = pending.intersection(&existing).collect();
    assert!(
        clash.is_empty(),
        "planned commands reuse existing ones: {clash:?}"
    );
}

#[test]
fn legacy_cookie_operations_are_not_required() {
    let spec = spec();
    let manifest = manifest();
    let spec_ops = spec_operations(&spec);

    for record in records(&manifest, "operations") {
        let id = text(record, "id");
        let security = &spec_ops[id].security;
        assert!(
            accepts_api_key(security),
            "{id} is counted as required but does not accept an API key"
        );
    }

    let mut cookie_only = 0;
    for record in records(&manifest, "excluded") {
        let id = text(record, "id");
        let security = &spec_ops[id].security;
        assert!(
            !mentions_scheme(security, "roblox-api-key"),
            "{id} wrongly excluded"
        );
        for key in ["status", "cli", "family"] {
            assert!(
                record.get(key).is_none(),
                "excluded operation {id} must not carry `{key}`"
            );
        }
        if mentions_scheme(security, "roblox-legacy-cookie") {
            cookie_only += 1;
            assert_eq!(text(record, "reason"), "legacy-cookie", "{id}");
        } else {
            assert_ne!(text(record, "reason"), "legacy-cookie", "{id}");
        }
    }
    assert!(cookie_only > 0, "spec snapshot has cookie-only operations");

    // Operations that accept either a cookie or an API key stay required.
    let both = records(&manifest, "operations")
        .iter()
        .filter(|record| {
            mentions_scheme(
                &spec_ops[text(record, "id")].security,
                "roblox-legacy-cookie",
            )
        })
        .count();
    assert!(both > 0);
}

#[test]
fn summary_matches_records() {
    let manifest = manifest();
    let operations = records(&manifest, "operations");
    let excluded = records(&manifest, "excluded");
    let count = |status: &str| {
        operations
            .iter()
            .filter(|record| text(record, "status") == status)
            .count() as u64
    };
    let summary = &manifest["summary"];
    assert_eq!(summary["api_key_operations"], operations.len() as u64);
    assert_eq!(summary["excluded"], excluded.len() as u64);
    assert_eq!(
        summary["spec_operations"],
        (operations.len() + excluded.len()) as u64
    );
    let mut covered = 0;
    for status in STATUSES {
        assert_eq!(summary[status], count(status), "summary.{status}");
        if COVERED.contains(&status) {
            covered += count(status);
        }
    }
    assert_eq!(summary["covered"], covered);
    assert_eq!(summary["not_covered"], operations.len() as u64 - covered);
    // A command that misses its documented path must never be counted as covered.
    assert!(!COVERED.contains(&"defective") && !COVERED.contains(&"pending"));

    let mut by_reason: BTreeMap<&str, u64> = BTreeMap::new();
    for record in excluded {
        *by_reason.entry(text(record, "reason")).or_default() += 1;
    }
    let listed = summary["excluded_by_reason"]
        .as_object()
        .expect("`excluded_by_reason` must be an object");
    assert_eq!(listed.len(), by_reason.len());
    for (reason, total) in by_reason {
        assert_eq!(listed[reason], total, "excluded_by_reason.{reason}");
    }
}

#[test]
fn status_matches_the_built_cli() {
    let manifest = manifest();
    // `existing`, `implemented` and `defective` owners are commands that exist today.
    let mut real = BTreeSet::new();
    let mut pending = BTreeSet::new();
    for record in records(&manifest, "operations") {
        let owners = if text(record, "status") == "pending" {
            &mut pending
        } else {
            &mut real
        };
        owners.insert(text(record, "cli"));
        if let Some(aliases) = record.get("cli_aliases") {
            owners.extend(aliases.as_array().unwrap().iter().filter_map(Value::as_str));
        }
    }
    for entry in records(&manifest, "outside_spec") {
        assert!(text(entry, "request").contains(" /"));
        real.insert(text(entry, "cli"));
    }
    assert!(!real.is_empty());

    for cli in real {
        assert!(
            help_succeeds(&cli_tokens(cli)),
            "`{cli}` is recorded as a real command but the CLI has no such command"
        );
    }
    for cli in pending {
        assert!(
            !help_succeeds(&cli_tokens(cli)),
            "`{cli}` exists in the CLI; set its status to `implemented` in openapi/coverage.json"
        );
    }
}

#[test]
fn spec_fields_match_the_snapshot() {
    let spec = spec();
    let manifest = manifest();
    let spec_ops = spec_operations(&spec);

    for record in records(&manifest, "operations") {
        let id = text(record, "id");
        let operation = &spec_ops[id];
        assert_eq!(text(record, "server"), operation.server, "{id}: server");
        assert_eq!(
            record["operation_id"], operation.operation_id,
            "{id}: operation_id"
        );
        assert_eq!(record["tags"], operation.tags, "{id}: tags");
        assert_eq!(record["stability"], operation.stability, "{id}: stability");
    }
    for key in ["operations", "excluded"] {
        for record in records(&manifest, key) {
            let id = text(record, "id");
            let auth: Vec<&str> = record["auth"]
                .as_array()
                .unwrap_or_else(|| panic!("{id}: `auth` must be an array"))
                .iter()
                .filter_map(Value::as_str)
                .collect();
            assert_eq!(auth, auth_names(&spec_ops[id].security), "{id}: auth");
        }
    }
}

/// `id` has no host. An endpoint is `server` + `path`, so that pair must be unique per method
/// and every server must be a Roblox HTTPS origin.
#[test]
fn server_and_id_locate_one_endpoint() {
    let manifest = manifest();
    let default_server = text(&manifest["spec"], "default_server");
    let mut endpoints = BTreeSet::new();
    let mut servers = BTreeSet::new();
    for record in records(&manifest, "operations") {
        let id = text(record, "id");
        let server = text(record, "server");
        let host = server
            .strip_prefix("https://")
            .unwrap_or_else(|| panic!("{id}: server `{server}` is not https"));
        assert!(
            host == "roblox.com" || host.ends_with(".roblox.com"),
            "{id}: server `{server}` is not a roblox.com origin"
        );
        assert!(
            !host.contains('/') && !host.contains('@') && !host.contains(':'),
            "{id}: server `{server}` must be a bare origin"
        );
        let url = format!(
            "{} {server}{}",
            text(record, "method"),
            text(record, "path")
        );
        assert!(
            endpoints.insert(url.clone()),
            "two records resolve to {url}"
        );
        servers.insert(server);
    }
    assert!(servers.contains(default_server));
    assert!(
        servers.len() > 1,
        "snapshot has operations off the default server; `server` must carry them"
    );
}

/// `summary` and docs/COVERAGE.md are derived from the manifest by scripts/gen_coverage.py.
/// Fails when either was edited by hand or not regenerated after a manifest change.
#[test]
fn generated_files_are_up_to_date() {
    let output = Command::new("python3")
        .arg("scripts/gen_coverage.py")
        .arg("--check")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("python3 is required to check docs/COVERAGE.md against openapi/coverage.json");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
