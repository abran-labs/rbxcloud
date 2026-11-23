//! Public CLI baseline: legacy commands keep their flags and `coverage` is read-only.
//! No network access and no credentials.

use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rbxcloud"))
        .args(args)
        .env_remove("RBXCLOUD_API_KEY")
        .output()
        .expect("run rbxcloud")
}

fn stdout(args: &[&str]) -> String {
    let out = run(args);
    assert!(out.status.success(), "{args:?} failed: {out:?}");
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn top_level_help_lists_legacy_commands_and_coverage() {
    let help = stdout(&["--help"]);
    for cmd in [
        "assets",
        "experience",
        "messaging",
        "datastore",
        "ordered-datastore",
        "inventory",
        "group",
        "luau",
        "subscription",
        "notification",
        "place",
        "universe",
        "user",
        "user-restriction",
        "coverage",
    ] {
        assert!(
            help.lines()
                .any(|l| l.trim_start().starts_with(cmd)
                    && l.trim_start()[cmd.len()..].starts_with(' ')),
            "missing `{cmd}` in help:\n{help}"
        );
    }
}

#[test]
fn assets_create_help_keeps_flags() {
    let help = stdout(&["assets", "create", "--help"]);
    for flag in [
        "--asset-type",
        "--display-name",
        "--description",
        "--expected-price",
        "--creator-id",
        "--creator-type",
        "--filepath",
        "--pretty",
        "--api-key",
    ] {
        assert!(help.contains(flag), "missing {flag}:\n{help}");
    }
}

#[test]
fn legacy_help_names_credential_env_var_without_printing_value() {
    let secret = "fake-help-contract-secret";
    for args in [
        ["assets", "create", "--help"],
        ["datastore", "get", "--help"],
        ["luau", "execute", "--help"],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_rbxcloud"))
            .args(args)
            .env("RBXCLOUD_API_KEY", secret)
            .output()
            .unwrap();
        assert!(out.status.success(), "{args:?}: {:?}", out.stderr);
        let help = String::from_utf8(out.stdout).unwrap();
        assert!(help.contains("RBXCLOUD_API_KEY"));
        assert!(!help.contains(secret), "{args:?} leaked key");
    }
}

#[test]
fn datastore_help_keeps_subcommands() {
    let help = stdout(&["datastore", "--help"]);
    for sub in [
        "list-stores",
        "list",
        "get",
        "set",
        "increment",
        "delete",
        "list-versions",
        "get-version",
    ] {
        assert!(
            help.lines()
                .any(|l| l.trim_start().starts_with(sub)
                    && l.trim_start()[sub.len()..].starts_with(' ')),
            "missing `{sub}`:\n{help}"
        );
    }
}

#[test]
fn coverage_reports_totals_by_family_without_credentials() {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../openapi/coverage.json")).unwrap();
    let ops = manifest["operations"].as_array().unwrap();
    let out = stdout(&["coverage"]);

    let header = out.lines().next().unwrap();
    for col in [
        "family",
        "implemented",
        "existing",
        "defective",
        "pending",
        "total",
    ] {
        assert!(header.contains(col), "header missing {col}: {header}");
    }
    let total = out
        .lines()
        .find(|l| l.split_whitespace().next() == Some("total"))
        .expect("total row");
    let cols: Vec<&str> = total.split_whitespace().collect();
    assert_eq!(cols.last().unwrap(), &ops.len().to_string());

    let mut families: Vec<&str> = ops.iter().map(|o| o["family"].as_str().unwrap()).collect();
    families.sort();
    families.dedup();
    for family in families {
        assert!(
            out.lines()
                .any(|l| l.split_whitespace().next() == Some(family)),
            "missing family {family}:\n{out}"
        );
    }
}

#[test]
fn coverage_is_read_only() {
    let help = stdout(&["coverage", "--help"]);
    for word in [
        "--api-key",
        "--url",
        "--method",
        "--body",
        "--path",
        "request",
    ] {
        assert!(!help.contains(word), "coverage exposes `{word}`:\n{help}");
    }
    let out = run(&["coverage", "GET", "/cloud/v2/anything"]);
    assert!(!out.status.success(), "coverage accepted positional args");
}
