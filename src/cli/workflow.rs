mod model;
mod target;

use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{bail, ensure, Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::Credentials,
};
use serde_json::{json, Value};

use model::{Document, Scope};

use super::asset_batch::WorkflowProject;

const PROJECT: &str = ".rbxcloud/workflows";
const PERSONAL: &str = "rbxcloud/workflows";
const MAX_SOURCE: u64 = 128 * 1024;
const MAX_CALLS: usize = 256;
const MAX_OUTPUT_BYTES: usize = 16 * 1024 * 1024;
const BLANK: &str = include_str!("../../workflows/blank.toml");
const UPLOAD_IMAGES: &str = include_str!("../../workflows/upload-images.toml");

#[derive(Debug, Args)]
pub struct Workflow {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create a workflow without replacing existing files
    Init {
        name: String,
        #[arg(long, value_enum, default_value = "blank")]
        template: Template,
        #[arg(long)]
        personal: bool,
    },
    /// List available project and personal workflows
    List,
    /// Print the selected workflow TOML
    Show { name: String },
    /// Check a workflow's schema, targets and references without executing it
    Validate { name: String },
    /// Evaluate a workflow in process using allowlisted commands
    Run {
        name: String,
        /// Bind a declared input; lists use JSON array syntax
        #[arg(long, value_name = "NAME=VALUE")]
        input: Vec<String>,
        /// Preflight locally; do not send requests or write a manifest
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum Template {
    Blank,
    UploadImages,
}

struct Roots {
    project: PathBuf,
    personal: Option<PathBuf>,
}

impl Roots {
    fn discover() -> Result<Self> {
        let cwd = env::current_dir()?.canonicalize()?;
        let project = cwd
            .ancestors()
            .find(|dir| dir.join(PROJECT).symlink_metadata().is_ok())
            .unwrap_or(&cwd)
            .to_owned();
        Ok(Self {
            project,
            personal: super::skill::config_home()
                .ok()
                .map(|path| path.join(PERSONAL)),
        })
    }

    fn project_store(&self) -> Result<PathBuf> {
        check_dir(&self.project.join(".rbxcloud"))?;
        let store = self.project.join(PROJECT);
        check_dir(&store)?;
        Ok(store)
    }

    fn stores(&self) -> Result<Vec<(&'static str, PathBuf)>> {
        let mut stores = vec![("project", self.project_store()?)];
        if let Some(store) = &self.personal {
            check_dir(store)?;
            stores.push(("personal", store.clone()));
        }
        Ok(stores)
    }

    fn locate(&self, name: &str) -> Result<PathBuf> {
        let file = workflow_file_name(name)?;
        for (_, store) in self.stores()? {
            let path = store.join(&file);
            match path.symlink_metadata() {
                Ok(meta) => {
                    ensure!(
                        meta.is_file(),
                        "workflow must be a regular file, not a symlink"
                    );
                    return Ok(path);
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
        bail!("workflow not found")
    }
}

fn check_dir(dir: &Path) -> Result<()> {
    match dir.symlink_metadata() {
        Ok(meta) => ensure!(
            meta.is_dir(),
            "workflow directory must be a directory, not a symlink"
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    Ok(())
}

fn workflow_file_name(name: &str) -> Result<String> {
    ensure!(
        model::identifier(name),
        "invalid workflow name (use lowercase ASCII, digits, '_' or '-')"
    );
    Ok(format!("{name}.toml"))
}

fn read_source(path: &Path) -> Result<String> {
    let mut file = fs::OpenOptions::new();
    file.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        file.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = file.open(path)?;
    ensure!(
        file.metadata()?.is_file(),
        "workflow must be a regular file"
    );
    let mut source = String::new();
    file.take(MAX_SOURCE + 1).read_to_string(&mut source)?;
    ensure!(
        source.len() as u64 <= MAX_SOURCE,
        "workflow TOML exceeds size limit"
    );
    Ok(source)
}

impl Workflow {
    pub async fn run(self) -> Result<Option<String>> {
        let roots = Roots::discover()?;
        let output = match self.command {
            Commands::Init {
                name,
                template,
                personal,
            } => {
                let file = workflow_file_name(&name)?;
                let dir = if personal {
                    let dir = roots
                        .personal
                        .as_ref()
                        .context("cannot locate personal config directory")?;
                    check_dir(dir)?;
                    dir.clone()
                } else {
                    roots.project_store()?
                };
                fs::create_dir_all(&dir)?;
                let template = match template {
                    Template::Blank => BLANK,
                    Template::UploadImages => UPLOAD_IMAGES,
                };
                let mut temp = tempfile::NamedTempFile::new_in(&dir)?;
                temp.write_all(template.as_bytes())?;
                let path = dir.join(file);
                temp.persist_noclobber(&path)
                    .context("cannot create workflow; existing files are not overwritten")?;
                format!("Created {}", path.display())
            }
            Commands::List => {
                let mut entries = Vec::new();
                let mut seen = BTreeSet::new();
                for (scope, dir) in roots.stores()? {
                    let dir = match fs::read_dir(dir) {
                        Ok(dir) => dir,
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                        Err(e) => return Err(e.into()),
                    };
                    let mut names = BTreeSet::new();
                    for entry in dir {
                        let entry = entry?;
                        if !entry.file_type()?.is_file() {
                            continue;
                        }
                        let file = entry.file_name();
                        if let Some(name) = file
                            .to_str()
                            .and_then(|s| s.strip_suffix(".toml"))
                            .filter(|s| model::identifier(s))
                        {
                            names.insert(name.to_owned());
                        }
                    }
                    entries.extend(names.into_iter().map(|name| {
                        let shadowed = if seen.insert(name.clone()) {
                            ""
                        } else {
                            ", shadowed"
                        };
                        format!("{name}\t({scope}{shadowed})")
                    }));
                }
                if entries.is_empty() {
                    "No workflows".to_owned()
                } else {
                    entries.join("\n")
                }
            }
            Commands::Show { name } => read_source(&roots.locate(&name)?)?
                .trim_end_matches('\n')
                .to_owned(),
            Commands::Validate { name } => {
                Document::parse(&read_source(&roots.locate(&name)?)?)?;
                "Workflow is valid (no commands executed)".into()
            }
            Commands::Run {
                name,
                input,
                dry_run,
            } => {
                let document = Document::parse(&read_source(&roots.locate(&name)?)?)?;
                let inputs = document.bind(&input)?;
                let project = WorkflowProject::new(&roots.project);
                let plan = preflight(&document, &inputs, &project)?;
                let result = if dry_run {
                    plan
                } else {
                    execute(&document, &inputs, &project, ClientPolicy::default(), None).await?
                };
                serde_json::to_string_pretty(&result)?
            }
        };
        Ok(Some(output))
    }
}

/// Offline evaluation deliberately does not invent prior command outputs.
fn preflight(
    document: &Document,
    inputs: &BTreeMap<String, Value>,
    project: &WorkflowProject<'_>,
) -> Result<Value> {
    let mut outputs = BTreeMap::new();
    let mut plan = Vec::new();
    let mut calls = 0;
    for step in &document.steps {
        let scope = Scope {
            inputs,
            outputs: &outputs,
            each: None,
        };
        let mut files = 0;
        let mut ready = 0;
        let mut deferred = 0;
        match step.iterations(scope)? {
            None => deferred += 1,
            Some(iterations) => {
                calls += iterations.len();
                ensure!(calls <= MAX_CALLS, "workflow iteration limit exceeded");
                for item in &iterations {
                    let scope = scope.with_each(item.as_ref());
                    match step.should_run(scope)? {
                        Some(false) => {}
                        None => deferred += 1,
                        Some(true) => match step.render(scope)? {
                            None => deferred += 1,
                            Some(action) => {
                                if let target::Action::Upload(batch) = action {
                                    files += batch.preflight_workflow(project).map_err(|_| {
                                        anyhow::anyhow!(
                                            "step {}: image input preflight failed (requires Linux openat2 and confined project paths)",
                                            step.id
                                        )
                                    })?;
                                }
                                ready += 1;
                            }
                        },
                    }
                }
            }
        }
        outputs.insert(step.id.clone(), None);
        plan.push(json!({"id": step.id, "command": step.command, "ready": ready, "deferred": deferred, "files": files}));
    }
    Ok(json!({"dryRun": true, "steps": plan}))
}

async fn execute(
    document: &Document,
    inputs: &BTreeMap<String, Value>,
    project: &WorkflowProject<'_>,
    policy: ClientPolicy,
    mut credentials: Option<Credentials>,
) -> Result<Value> {
    let mut outputs = BTreeMap::new();
    let mut clients = BTreeMap::<u8, AsyncClient>::new();
    let mut calls = 0;
    let mut output_bytes = 0;
    for step in &document.steps {
        let scope = Scope {
            inputs,
            outputs: &outputs,
            each: None,
        };
        let iterations = step
            .iterations(scope)?
            .context("step iteration input is unavailable")?;
        calls += iterations.len();
        ensure!(calls <= MAX_CALLS, "workflow iteration limit exceeded");
        let mut results = Vec::new();
        for item in &iterations {
            let scope = scope.with_each(item.as_ref());
            if !step
                .should_run(scope)?
                .context("step condition is unavailable")?
            {
                continue;
            }
            let action = step
                .render(scope)?
                .context("step command input is unavailable")?;
            let transport = match clients.entry(step.retries) {
                std::collections::btree_map::Entry::Occupied(client) => client.get().clone(),
                std::collections::btree_map::Entry::Vacant(empty) => {
                    let credential = match &mut credentials {
                        Some(c) => c,
                        None => credentials.insert(Credentials::from_process()?),
                    };
                    empty
                        .insert(
                            AsyncClient::new(
                                credential.clone(),
                                policy
                                    .clone()
                                    .max_retries(step.retries.into())
                                    .max_response_bytes(2 * 1024 * 1024),
                            )
                            .await?,
                        )
                        .clone()
                }
            };
            let output = match action {
                target::Action::Upload(batch) => batch.run_workflow(project, transport).await
                    .map_err(|_| anyhow::anyhow!("step {}: batch failed or unresolved; inspect the project upload manifest", step.id))?,
                read => read.read(transport).await.map_err(|err| anyhow::anyhow!("step {}: read failed{}",
                    step.id, err.status().map(|status| format!(" (HTTP {status})")).unwrap_or_default()))?,
            };
            output_bytes += serde_json::to_vec(&output)?.len();
            ensure!(
                output_bytes <= MAX_OUTPUT_BYTES,
                "workflow exceeded its output size limit"
            );
            results.push(output);
        }
        let output = if step.for_each.is_some() {
            Value::Array(results)
        } else {
            results.pop().unwrap_or(Value::Null)
        };
        outputs.insert(step.id.clone(), Some(output));
    }
    Ok(json!({"dryRun": false, "outputs": outputs}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{Reply, TestServer};
    use rbxcloud::rbx::credentials::SecretString;

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn upload_template_records_and_reuses_project_manifest() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("icons/sub")).unwrap();
        fs::write(root.path().join("icons/sub/a.png"), b"image fixture").unwrap();
        let document = Document::parse(UPLOAD_IMAGES).unwrap();
        let inputs = document.bind(&["creator_id=42".into()]).unwrap();
        let project = WorkflowProject::new(root.path());
        let server = TestServer::start(vec![Reply::json(
            200,
            r#"{"done":true,"response":{"assetId":"123"}}"#,
        )]);
        let plan = preflight(&document, &inputs, &project).unwrap();
        assert_eq!(plan["steps"][0]["files"], 1);
        assert!(!root.path().join(".rbxcloud").exists());
        for _ in 0..2 {
            let result = execute(
                &document,
                &inputs,
                &project,
                ClientPolicy::loopback(server.addr()).unwrap(),
                Some(Credentials::api_key(SecretString::new("workflow-test-key"))),
            )
            .await
            .unwrap();
            let entry = &result["outputs"]["images"]["entries"][0];
            assert_eq!(entry["result"]["asset_id"], 123);
            assert!(entry.get("path").is_none());
        }
        let state = fs::read_to_string(root.path().join(".rbxcloud/upload-batch.json")).unwrap();
        assert!(state.contains("\"asset_id\": 123"));
        assert_eq!(server.requests().len(), 1);
        assert_eq!(server.requests()[0].method, "POST");
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn upload_template_refuses_symlinked_state_entries_without_requests() {
        use std::os::unix::fs::symlink;

        for entry in [
            ".rbxcloud",
            ".rbxcloud/upload-batch.json",
            ".rbxcloud/upload-batch.json.lock",
        ] {
            let root = tempfile::tempdir().unwrap();
            fs::create_dir(root.path().join("icons")).unwrap();
            fs::write(root.path().join("icons/a.png"), b"image fixture").unwrap();
            let entry = root.path().join(entry);
            let fixture = root.path().join("fixture");
            if entry == root.path().join(".rbxcloud") {
                fs::create_dir(&fixture).unwrap();
            } else {
                fs::create_dir(entry.parent().unwrap()).unwrap();
                fs::write(&fixture, br#"{"version":1,"entries":{}}"#).unwrap();
            }
            symlink(&fixture, &entry).unwrap();
            let document = Document::parse(UPLOAD_IMAGES).unwrap();
            let inputs = document.bind(&["creator_id=42".into()]).unwrap();
            let server = TestServer::start(vec![]);
            assert!(execute(
                &document,
                &inputs,
                &WorkflowProject::new(root.path()),
                ClientPolicy::loopback(server.addr()).unwrap(),
                Some(Credentials::api_key(SecretString::new("workflow-test-key"))),
            )
            .await
            .is_err());
            assert!(server.requests().is_empty());
            assert_eq!(fs::read_link(entry).unwrap(), fixture);
        }
    }

    #[tokio::test]
    async fn loops_over_json_output_gates_on_booleans_and_retries_one_safe_read() {
        let root = tempfile::tempdir().unwrap();
        let document = Document::parse(
            r#"
version = 1
[inputs.asset_id]
type = "integer"
default = 17

[[steps]]
id = "versions"
command = ["asset-tools", "list-asset-versions"]
args = ["--asset-id", "${inputs.asset_id}"]
retries = 1

[[steps]]
id = "published"
command = ["asset-tools", "get-asset-version"]
args = ["--asset-id", "${inputs.asset_id}", "--version-number", "${each.creationContext.version}"]
for_each = "${steps.versions.output.assetVersions}"
when = "${each.published}"
"#,
        )
        .unwrap();
        let inputs = document.bind(&[]).unwrap();
        let project = WorkflowProject::new(root.path());
        let plan = preflight(&document, &inputs, &project).unwrap();
        assert_eq!(plan["steps"][0]["ready"], 1);
        assert_eq!(plan["steps"][1]["deferred"], 1);
        let server = TestServer::start(vec![
            Reply::empty(429).header("Retry-After", "0"),
            Reply::json(
                200,
                r#"{"assetVersions":[{"published":false},{"published":true,"creationContext":{"version":3}}]}"#,
            ),
            Reply::json(
                200,
                r#"{"path":"assets/17/versions/3","published":true,"creationContext":{"note":"workflow-test-key"}}"#,
            ),
        ]);
        let policy = ClientPolicy::loopback(server.addr()).unwrap();
        let credentials = Credentials::api_key(SecretString::new("workflow-test-key"));
        let result = execute(&document, &inputs, &project, policy, Some(credentials))
            .await
            .unwrap();
        assert_eq!(
            result["outputs"]["published"][0]["path"],
            "assets/17/versions/3"
        );
        assert_eq!(result["outputs"]["published"].as_array().unwrap().len(), 1);
        assert!(!result.to_string().contains("workflow-test-key"));
        assert_eq!(
            server
                .requests()
                .iter()
                .map(|r| r.target.as_str())
                .collect::<Vec<_>>(),
            [
                "/assets/v1/assets/17/versions",
                "/assets/v1/assets/17/versions",
                "/assets/v1/assets/17/versions/3",
            ]
        );
        assert!(server.requests().iter().all(|r| r.method == "GET"));
    }

    #[tokio::test]
    async fn one_retry_does_not_repeat_failed_read_again() {
        let root = tempfile::tempdir().unwrap();
        let document = Document::parse(
            r#"
version = 1
[[steps]]
id = "versions"
command = ["asset-tools", "list-asset-versions"]
args = ["--asset-id", "17"]
retries = 1
"#,
        )
        .unwrap();
        let server = TestServer::start(vec![Reply::empty(503).header("Retry-After", "0"); 2]);
        let policy = ClientPolicy::loopback(server.addr()).unwrap();
        let credentials = Credentials::api_key(SecretString::new("workflow-test-key"));
        assert!(execute(
            &document,
            &document.bind(&[]).unwrap(),
            &WorkflowProject::new(root.path()),
            policy,
            Some(credentials)
        )
        .await
        .is_err());
        assert_eq!(server.requests().len(), 2);
    }
}
