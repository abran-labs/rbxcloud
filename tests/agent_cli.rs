use std::{fs, path::Path, process::Command};

fn cli(home: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rbxcloud"))
        .current_dir(home)
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join("xdg"))
        .env_remove("ROBLOX_OPEN_CLOUD_API_KEY")
        .env_remove("RBXCLOUD_API_KEY")
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn batch_preview_resolves_directory_and_glob_without_credentials_or_writes() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path();
    fs::create_dir_all(home.join("icons/sub")).unwrap();
    fs::write(home.join("icons/a.png"), b"abc").unwrap();
    fs::write(home.join("icons/sub/b.png"), b"def").unwrap();
    fs::write(home.join("icons/note.txt"), b"not an image").unwrap();
    let output = cli(
        home,
        &[
            "assets",
            "upload-batch",
            "--dry-run",
            "--creator-type",
            "group",
            "--creator-id",
            "42",
            "--asset-type",
            "image",
            "icons",
            "icons/**/*.png",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let preview: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let entries = preview["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(
        entries[0]["sha256"],
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(preview["creatorId"], 42);
    assert!(!home.join(".rbxcloud").exists());
}

#[test]
fn skill_install_preserves_local_edits_and_matches_binary_version() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path();
    let printed = cli(home, &["skill", "install", "--print"]);
    assert!(
        printed.status.success(),
        "{}",
        String::from_utf8_lossy(&printed.stderr)
    );
    assert!(String::from_utf8_lossy(&printed.stdout).contains(env!("CARGO_PKG_VERSION")));
    assert!(!home.join(".agents").exists());
    for (flag, root) in [
        (None, ".agents/skills"),
        (Some("--claude"), ".claude/skills"),
        (Some("--opencode"), "xdg/opencode/skills"),
    ] {
        let mut args = vec!["skill", "install"];
        args.extend(flag);
        assert!(cli(home, &args).status.success());
        let installed = home.join(root).join("roblox-open-cloud/SKILL.md");
        assert_eq!(fs::read(&installed).unwrap(), printed.stdout);
        fs::write(&installed, b"local edits").unwrap();
        assert!(!cli(home, &args).status.success());
        assert_eq!(fs::read(&installed).unwrap(), b"local edits");
        args.push("--force");
        assert!(cli(home, &args).status.success());
        assert_eq!(fs::read(&installed).unwrap(), printed.stdout);
    }
}

#[cfg(unix)]
#[test]
fn auth_status_checks_real_loader_without_disclosing_key() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path();
    assert!(!cli(home, &["auth", "status"]).status.success());
    let secret = "fake-contract-key-not-for-output";
    let file = home.join(".config/opencode/secrets/roblox-open-cloud.env");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, format!("ROBLOX_OPEN_CLOUD_API_KEY={secret}\n")).unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(!cli(home, &["auth", "status"]).status.success());
    fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
    let output = cli(home, &["auth", "status"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    for bytes in [&output.stdout, &output.stderr] {
        assert!(!String::from_utf8_lossy(bytes).contains(secret));
    }
}

#[test]
fn auth_status_requires_absolute_home_only_for_file_fallback() {
    let temp = tempfile::tempdir().unwrap();
    for home in [None, Some(""), Some("relative")] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_rbxcloud"));
        command
            .current_dir(temp.path())
            .env_remove("HOME")
            .env_remove("USERPROFILE")
            .env_remove("ROBLOX_OPEN_CLOUD_API_KEY")
            .env_remove("RBXCLOUD_API_KEY")
            .args(["auth", "status"]);
        if let Some(home) = home {
            command.env("HOME", home);
        }
        let missing = command.output().unwrap();
        assert!(!missing.status.success());
        assert!(String::from_utf8_lossy(&missing.stderr).contains("absolute home directory"));
        assert!(command
            .env("ROBLOX_OPEN_CLOUD_API_KEY", "fake-environment-only")
            .output()
            .unwrap()
            .status
            .success());
    }
}

#[cfg(unix)]
#[test]
fn batch_rejects_non_utf8_parent_before_upload_setup() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join(OsStr::from_bytes(b"icons-\xff"));
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("ok.png"), b"image fixture").unwrap();
    let output = cli(
        temp.path(),
        &[
            "assets",
            "upload-batch",
            "--creator-type",
            "user",
            "--creator-id",
            "42",
            "--asset-type",
            "image",
            ".",
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("non-UTF-8 upload path"));
    assert!(!temp.path().join(".rbxcloud").exists());
}

#[cfg(target_os = "linux")]
#[test]
fn workflow_preflight_rejects_even_in_project_directory_aliases() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path();
    fs::create_dir(home.join("icons")).unwrap();
    fs::write(home.join("icons/a.png"), b"image fixture").unwrap();
    std::os::unix::fs::symlink("icons", home.join("alias")).unwrap();
    assert!(cli(
        home,
        &["workflow", "init", "icons", "--template", "upload-images"]
    )
    .status
    .success());
    let output = cli(
        home,
        &[
            "workflow",
            "run",
            "icons",
            "--input",
            "creator_id=42",
            "--input",
            r#"paths=["alias/a.png"]"#,
            "--dry-run",
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("image input preflight failed"));
}

#[cfg(target_os = "linux")]
#[test]
fn workflow_dry_run_preflights_batch_without_mutations() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path();
    assert!(cli(
        home,
        &["workflow", "init", "icons", "--template", "upload-images"]
    )
    .status
    .success());
    fs::create_dir(home.join("icons")).unwrap();
    fs::write(home.join("icons/a.png"), b"abc").unwrap();
    let output = cli(
        home,
        &[
            "workflow",
            "run",
            "icons",
            "--input",
            "creator_id=42",
            "--dry-run",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["dryRun"], true);
    assert_eq!(plan["steps"][0]["files"], 1);
    assert!(!home.join(".rbxcloud/upload-batch.json").exists());
}

#[test]
fn workflows_reject_unsafe_targets_and_rendered_values_even_before_network() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path();
    let dir = home.join(".rbxcloud/workflows");
    fs::create_dir_all(&dir).unwrap();
    for target in [
        r#"["luau", "run"]"#,
        r#"["place", "publish"]"#,
        r#"["place-instance-management", "update-instance"]"#,
        r#"["skill", "install"]"#,
        r#"["workflow", "run"]"#,
        r#"["asset-tools", "rollback-asset-version"]"#,
        r#"["asset-download", "get"]"#,
        r#"["${inputs.command}", "run"]"#,
    ] {
        fs::write(
            dir.join("bad.toml"),
            format!("version = 1\n[[steps]]\nid = 'bad'\ncommand = {target}\nwhen = false\n"),
        )
        .unwrap();
        let result = cli(home, &["workflow", "validate", "bad"]);
        assert!(!result.status.success(), "{target}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("not allowlisted"));
    }
    assert!(cli(
        home,
        &["workflow", "init", "icons", "--template", "upload-images"]
    )
    .status
    .success());
    let injected = r#"paths=["--api-key=must-never-echo-this"]"#;
    let output = cli(
        home,
        &[
            "workflow",
            "run",
            "icons",
            "--input",
            "creator_id=42",
            "--input",
            injected,
            "--dry-run",
        ],
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not a flag"));
    assert!(!stderr.contains("must-never-echo-this"));
    for args in [
        r#"["--asset-id", "1", "--api-key", "not-a-key"]"#,
        r#"["${inputs.flag}", "1"]"#,
    ] {
        fs::write(
            dir.join("bad.toml"),
            format!("version = 1\n[inputs.flag]\ntype = 'string'\n[[steps]]\nid = 'bad'\ncommand = ['asset-tools','list-asset-versions']\nargs = {args}\n"),
        ).unwrap();
        assert!(!cli(home, &["workflow", "validate", "bad"]).status.success());
    }
}

#[test]
fn project_workflow_shadows_personal_and_init_never_clobbers() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path();
    assert!(cli(home, &["workflow", "init", "icons", "--personal"])
        .status
        .success());
    assert!(home.join("xdg/rbxcloud/workflows/icons.toml").exists());
    assert!(cli(
        home,
        &["workflow", "init", "icons", "--template", "upload-images"]
    )
    .status
    .success());
    assert!(!cli(home, &["workflow", "init", "icons"]).status.success());
    let listed = cli(home, &["workflow", "list"]);
    assert!(listed.status.success());
    assert_eq!(
        String::from_utf8_lossy(&listed.stdout),
        "icons\t(project)\nicons\t(personal, shadowed)\n"
    );
    let shown = cli(home, &["workflow", "show", "icons"]);
    assert!(String::from_utf8_lossy(&shown.stdout).contains("upload-batch"));
}

#[cfg(unix)]
#[test]
fn workflow_rejects_asset_paths_outside_the_project_even_through_symlink() {
    let project = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("one.png"), b"outside the project").unwrap();
    std::os::unix::fs::symlink(outside.path(), project.path().join("icons")).unwrap();
    assert!(cli(
        project.path(),
        &["workflow", "init", "icons", "--template", "upload-images"]
    )
    .status
    .success());
    let output = cli(
        project.path(),
        &[
            "workflow",
            "run",
            "icons",
            "--input",
            "creator_id=42",
            "--dry-run",
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("image input preflight failed"));
}

#[cfg(unix)]
#[test]
fn glob_ignores_non_utf8_names_and_never_walks_directory_symlinks() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
    let home = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    fs::create_dir(home.path().join("icons")).unwrap();
    fs::write(home.path().join("icons/ok.png"), b"valid input").unwrap();
    fs::write(
        home.path()
            .join("icons")
            .join(OsStr::from_bytes(b"\xff.png")),
        b"invalid filename",
    )
    .unwrap();
    fs::write(outside.path().join("outside.png"), b"do not traverse").unwrap();
    std::os::unix::fs::symlink(outside.path(), home.path().join("icons/link")).unwrap();
    let output = cli(
        home.path(),
        &[
            "assets",
            "upload-batch",
            "--dry-run",
            "--creator-type",
            "user",
            "--creator-id",
            "42",
            "--asset-type",
            "image",
            "icons/**/*.png",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["entries"].as_array().unwrap().len(), 1);
    assert!(plan["entries"][0]["path"]
        .as_str()
        .unwrap()
        .ends_with("ok.png"));
}
