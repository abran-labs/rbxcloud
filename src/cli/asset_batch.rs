use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    time::Duration,
};

use anyhow::{bail, ensure, Context, Result};
use clap::Args;
use futures_util::{stream, StreamExt};
use rbxcloud::rbx::{
    asset_create::{AssetCreateClient, AssetKind, AssetOperation, CreateAsset, CreatorKind},
    client::{AsyncClient, ClientPolicy, Error as TransportError},
    credentials::Credentials,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const DEFAULT_MANIFEST: &str = ".rbxcloud/upload-batch.json";
const MAX_BYTES: u64 = 20_000_000;
const MAX_ENTRIES: usize = 10_000;
const MAX_GLOB_NODES: usize = 100_000;

#[derive(Debug, Args)]
pub struct BatchArgs {
    /// Files, directories (recursive), or quoted glob patterns
    #[arg(required = true, num_args = 1..)]
    paths: Vec<String>,
    #[arg(long)]
    creator_type: CreatorKind,
    #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
    creator_id: u64,
    #[arg(long)]
    asset_type: AssetKind,
    /// Default: .rbxcloud/upload-batch.json
    #[arg(long, default_value = DEFAULT_MANIFEST)]
    manifest: PathBuf,
    #[arg(long, default_value_t = 2, value_parser = clap::value_parser!(u8).range(1..=8))]
    concurrency: u8,
    /// Inspect files and hashes locally without credentials or manifest writes
    #[arg(long)]
    dry_run: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    path: PathBuf,
    sha256: String,
    #[serde(skip)]
    key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
enum State {
    // This is durable BEFORE sending POST; after any uncertain outcome, never replay.
    Unconfirmed,
    Accepted { operation_id: String },
    Created { asset_id: u64 },
    Failed { status: Option<u16> },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u8,
    entries: BTreeMap<String, State>,
}

struct LockedManifest {
    _lock: File,
    path: PathBuf,
    data: Manifest,
}

impl LockedManifest {
    fn open(path: &Path) -> Result<Self> {
        let name = path.file_name().context("manifest must name a file")?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        let parent = parent.canonicalize()?;
        let path = parent.join(name);
        let mut lock_name = name.to_os_string();
        lock_name.push(".lock");
        let mut opts = OpenOptions::new();
        opts.create(true).read(true).write(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        let lock_path = parent.join(lock_name);
        if let Ok(meta) = lock_path.symlink_metadata() {
            ensure!(
                meta.file_type().is_file(),
                "manifest lock must be a regular file"
            );
        }
        let lock = opts.open(lock_path)?;
        ensure!(
            lock.metadata()?.is_file(),
            "manifest lock must be a regular file"
        );
        fs2::FileExt::try_lock_exclusive(&lock).context("cannot acquire batch manifest lock")?;
        let data = match path.symlink_metadata() {
            Ok(meta) => {
                ensure!(
                    meta.file_type().is_file(),
                    "manifest must be a regular file"
                );
                serde_json::from_reader(std::io::BufReader::new(File::open(&path)?))
                    .context("invalid batch manifest")?
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Manifest {
                version: 1,
                entries: BTreeMap::new(),
            },
            Err(e) => return Err(e.into()),
        };
        ensure!(data.version == 1, "unsupported manifest version");
        Ok(Self {
            _lock: lock,
            path,
            data,
        })
    }

    fn set(&mut self, key: String, state: State) -> Result<()> {
        self.data.entries.insert(key, state);
        let parent = self.path.parent().context("manifest has no parent")?;
        let mut temp = tempfile::NamedTempFile::new_in(parent)?;
        serde_json::to_writer_pretty(&mut temp, &self.data)?;
        temp.write_all(b"\n")?;
        temp.as_file().sync_all()?;
        temp.persist(&self.path)
            .context("cannot replace batch manifest")?;
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        Ok(())
    }
}

impl BatchArgs {
    pub(super) async fn run(self) -> Result<String> {
        let entries = self.prepare(None)?;
        if self.dry_run {
            return self.preview(&entries);
        }
        let transport =
            AsyncClient::new(Credentials::from_process()?, ClientPolicy::default()).await?;
        self.upload(
            entries,
            AssetCreateClient::new(transport),
            Duration::from_secs(1),
        )
        .await
    }

    pub(super) fn preflight_workflow(&self, root: &Path) -> Result<usize> {
        self.prepare(Some(root)).map(|entries| entries.len())
    }

    pub(super) async fn run_workflow(
        mut self,
        root: &Path,
        transport: AsyncClient,
    ) -> Result<serde_json::Value> {
        ensure!(
            matches!(self.asset_type, AssetKind::Image),
            "workflows can only upload images"
        );
        let entries = self.prepare(Some(root))?;
        let state_dir = root.join(".rbxcloud");
        match state_dir.symlink_metadata() {
            Ok(meta) => ensure!(
                meta.is_dir(),
                "project .rbxcloud must not be a symlink or special file"
            ),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => fs::create_dir(&state_dir)?,
            Err(e) => return Err(e.into()),
        }
        self.manifest = root.join(DEFAULT_MANIFEST);
        let result = self
            .upload(
                entries,
                AssetCreateClient::new(transport),
                Duration::from_secs(1),
            )
            .await?;
        let mut value: serde_json::Value = serde_json::from_str(&result)?;
        for entry in value["entries"]
            .as_array_mut()
            .context("invalid batch result")?
        {
            entry
                .as_object_mut()
                .context("invalid batch entry")?
                .remove("path");
        }
        Ok(value)
    }

    fn preview(&self, entries: &[Entry]) -> Result<String> {
        Ok(serde_json::to_string_pretty(&serde_json::json!({
            "dryRun": true,
            "creatorType": self.creator_type,
            "creatorId": self.creator_id,
            "assetType": self.asset_type,
            "entries": entries,
        }))?)
    }

    // Direct CLI paths may be anywhere. Workflow uploads get an explicit project root.
    fn prepare(&self, root: Option<&Path>) -> Result<Vec<Entry>> {
        let mut files = BTreeSet::new();
        for pattern in &self.paths {
            let input = Path::new(pattern);
            let input = if let Some(root) = root {
                ensure!(
                    input.components().all(|c| matches!(
                        c,
                        std::path::Component::Normal(_) | std::path::Component::CurDir
                    )),
                    "workflow asset paths must be project-relative without parent segments"
                );
                root.join(input)
            } else {
                input.to_path_buf()
            };
            match input.symlink_metadata() {
                Ok(meta) if meta.is_file() => {
                    ensure!(
                        self.supported(&input),
                        "file extension does not match asset type"
                    );
                    insert_file(&mut files, &input, root)?;
                }
                Ok(meta) if meta.is_dir() => self.walk(input, &mut files, root)?,
                Ok(_) => bail!("explicit upload paths must be regular files or directories"),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    self.expand_glob(Path::new(pattern), &mut files, root)?;
                }
                Err(e) => return Err(e.into()),
            }
        }
        ensure!(!files.is_empty(), "no supported files to upload");
        let mut entries = Vec::with_capacity(files.len());
        for path in files {
            let name = file_name(&path)?;
            ensure!(
                name.len() <= 255
                    && !name.trim().is_empty()
                    && !name.contains(['"', '\\'])
                    && !name
                        .chars()
                        .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}')),
                "invalid upload file name"
            );
            let sha256 = hash(&read_file(&path)?);
            let key = format!(
                "{:?}:{}:{:?}:{sha256}",
                self.creator_type, self.creator_id, self.asset_type
            );
            entries.push(Entry { path, sha256, key });
        }
        Ok(entries)
    }

    fn supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| self.asset_type.content_type(ext))
            .is_some()
    }

    fn expand_glob(
        &self,
        pattern: &Path,
        files: &mut BTreeSet<PathBuf>,
        root: Option<&Path>,
    ) -> Result<()> {
        let mut base = root.unwrap_or(Path::new(".")).to_path_buf();
        let mut segments = Vec::new();
        for part in pattern.components() {
            match part {
                Component::Normal(segment) => {
                    let segment = segment.to_str().context("non-UTF-8 glob segment")?;
                    if segments.is_empty() && !segment.contains(['*', '?', '[']) {
                        base.push(segment);
                        if !base.symlink_metadata().is_ok_and(|m| m.is_dir()) {
                            bail!("upload path/glob matched no supported regular files");
                        }
                    } else {
                        segments.push(glob::Pattern::new(segment)?);
                    }
                }
                Component::Prefix(_) | Component::RootDir | Component::ParentDir
                    if root.is_none() && segments.is_empty() =>
                {
                    base.push(part.as_os_str())
                }
                Component::CurDir => {}
                _ => bail!("invalid glob path component"),
            }
        }
        let options = glob::MatchOptions {
            require_literal_separator: true,
            require_literal_leading_dot: true,
            ..glob::MatchOptions::new()
        };
        let mut visited = BTreeSet::new();
        let mut todo = vec![(base, 0)];
        let mut matched = false;
        let mut inspected = 0;
        while let Some((dir, index)) = todo.pop() {
            if !visited.insert((dir.clone(), index)) {
                continue;
            }
            let Some(segment) = segments.get(index) else {
                continue;
            };
            let recursive = segment.as_str() == "**";
            let last = index + 1 == segments.len();
            if recursive && !last {
                todo.push((dir.clone(), index + 1));
            }
            for item in fs::read_dir(&dir)? {
                inspected += 1;
                ensure!(
                    inspected <= MAX_GLOB_NODES,
                    "upload glob exceeded its traversal limit"
                );
                let item = item?;
                let name = item.file_name();
                let Some(name) = name.to_str() else { continue };
                let matches = if recursive {
                    !name.starts_with('.')
                } else {
                    segment.matches_with(name, options)
                };
                if !matches {
                    continue;
                }
                let file_type = item.file_type()?;
                if file_type.is_dir() && (recursive || !last) {
                    todo.push((item.path(), if recursive { index } else { index + 1 }));
                } else if last && file_type.is_file() && self.supported(&item.path()) {
                    insert_file(files, &item.path(), root)?;
                    matched = true;
                }
            }
        }
        ensure!(
            matched,
            "upload path/glob matched no supported regular files"
        );
        Ok(())
    }

    fn walk(
        &self,
        path: PathBuf,
        files: &mut BTreeSet<PathBuf>,
        root: Option<&Path>,
    ) -> Result<()> {
        let mut dirs = vec![path];
        while let Some(dir) = dirs.pop() {
            if let Some(root) = root {
                ensure!(
                    dir.canonicalize()?.starts_with(root),
                    "asset directory escapes project"
                );
            }
            for item in fs::read_dir(dir)? {
                let item = item?;
                let ty = item.file_type()?;
                if ty.is_dir() {
                    dirs.push(item.path());
                } else if ty.is_file()
                    && item.file_name().to_str().is_some()
                    && self.supported(&item.path())
                {
                    insert_file(files, &item.path(), root)?;
                }
            }
        }
        Ok(())
    }

    async fn upload(
        &self,
        entries: Vec<Entry>,
        client: AssetCreateClient,
        poll_delay: Duration,
    ) -> Result<String> {
        let manifest = tokio::sync::Mutex::new(LockedManifest::open(&self.manifest)?);
        let mut unique = BTreeMap::new();
        for entry in &entries {
            unique.entry(&entry.key).or_insert(entry);
        }
        let results: Vec<_> = stream::iter(
            unique
                .into_values()
                .map(|entry| self.process(entry, &manifest, &client, poll_delay)),
        )
        .buffer_unordered(self.concurrency.into())
        // Finish active requests even if another manifest write fails.
        .collect()
        .await;
        for result in results {
            result?;
        }
        let manifest = manifest.into_inner();
        let mut unresolved = 0;
        let result: Vec<_> = entries
            .into_iter()
            .map(|entry| {
                let state = &manifest.data.entries[&entry.key];
                if !matches!(state, State::Created { .. }) {
                    unresolved += 1;
                }
                serde_json::json!({
                    "path": entry.path,
                    "sha256": entry.sha256,
                    "result": state,
                })
            })
            .collect();
        ensure!(
            unresolved == 0,
            "{unresolved} upload path(s) unresolved; inspect manifest. Resume with the same manifest. Unconfirmed uploads never resend."
        );
        Ok(serde_json::to_string_pretty(
            &serde_json::json!({"entries": result}),
        )?)
    }

    async fn process(
        &self,
        entry: &Entry,
        manifest: &tokio::sync::Mutex<LockedManifest>,
        client: &AssetCreateClient,
        poll_delay: Duration,
    ) -> Result<()> {
        let existing = manifest.lock().await.data.entries.get(&entry.key).cloned();
        let operation_id = match existing {
            Some(State::Accepted { operation_id }) => operation_id,
            Some(_) => return Ok(()),
            None => {
                let data = read_file(&entry.path)?;
                ensure!(
                    hash(&data) == entry.sha256,
                    "asset changed after planning; rerun batch"
                );
                let name = file_name(&entry.path)?;
                let display_name = entry
                    .path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .context("invalid asset name")?;
                manifest
                    .lock()
                    .await
                    .set(entry.key.clone(), State::Unconfirmed)?;
                let op = match client
                    .create(CreateAsset {
                        kind: self.asset_type,
                        creator: self.creator_type,
                        creator_id: self.creator_id,
                        display_name,
                        description: "",
                        file_name: name,
                        data,
                    })
                    .await
                {
                    Ok(op) => op,
                    Err(err) => {
                        // Only documented explicit rejections are terminal rather than ambiguous.
                        if let TransportError::Status {
                            status: status @ (400 | 401 | 403),
                            ..
                        } = err
                        {
                            manifest.lock().await.set(
                                entry.key.clone(),
                                State::Failed {
                                    status: Some(status),
                                },
                            )?;
                        }
                        return Ok(());
                    }
                };
                if let Some(state) = terminal(&op) {
                    return manifest.lock().await.set(entry.key.clone(), state);
                }
                let Some(operation_id) = accepted_id(&op) else {
                    return Ok(());
                };
                manifest.lock().await.set(
                    entry.key.clone(),
                    State::Accepted {
                        operation_id: operation_id.clone(),
                    },
                )?;
                operation_id
            }
        };
        ensure!(
            valid_operation_id(&operation_id),
            "manifest contains an invalid operation ID"
        );
        for check in 0..8 {
            if check > 0 {
                tokio::time::sleep(poll_delay).await;
            }
            let Ok(op) = client.operation(&operation_id).await else {
                break;
            };
            if (op.path.is_some() || op.operation_id.is_some())
                && accepted_id(&op).as_deref() != Some(&operation_id)
            {
                break;
            }
            if let Some(state) = terminal(&op) {
                return manifest.lock().await.set(entry.key.clone(), state);
            }
        }
        // Retain the known ID even if GET failed or the operation remains pending.
        Ok(())
    }
}

fn insert_file(files: &mut BTreeSet<PathBuf>, path: &Path, root: Option<&Path>) -> Result<()> {
    let path = path.canonicalize()?;
    if let Some(root) = root {
        ensure!(path.starts_with(root), "asset file escapes project");
    }
    files.insert(path);
    ensure!(files.len() <= MAX_ENTRIES, "batch contains too many files");
    Ok(())
}

fn file_name(path: &Path) -> Result<&str> {
    path.file_name()
        .and_then(|n| n.to_str())
        .context("non-UTF-8 upload file name")
}

fn read_file(path: &Path) -> Result<Vec<u8>> {
    let mut opts = OpenOptions::new();
    opts.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = opts.open(path)?;
    ensure!(
        file.metadata()?.is_file(),
        "upload input must remain a regular file"
    );
    let mut data = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut data)?;
    ensure!(
        !data.is_empty() && data.len() as u64 <= MAX_BYTES,
        "upload file must be 1 to {MAX_BYTES} bytes"
    );
    Ok(data)
}

fn hash(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

fn valid_operation_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 256
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}

fn accepted_id(op: &AssetOperation) -> Option<String> {
    let path_id = match &op.path {
        Some(path) => Some(path.strip_prefix("operations/")?),
        None => None,
    };
    let direct = op.operation_id.as_deref();
    if path_id.zip(direct).is_some_and(|(a, b)| a != b) {
        return None;
    }
    path_id
        .or(direct)
        .filter(|id| valid_operation_id(id))
        .map(str::to_owned)
}

fn terminal(op: &AssetOperation) -> Option<State> {
    if !op.done {
        return None;
    }
    if op.error.is_some() {
        return Some(State::Failed { status: None });
    }
    fn asset_id(value: &str) -> Option<u64> {
        value.parse().ok().filter(|id| *id > 0)
    }
    let response = op.response.as_ref()?;
    let direct = response.asset_id.as_deref().map(asset_id);
    let path = response
        .path
        .as_deref()
        .map(|path| asset_id(path.strip_prefix("assets/")?));
    let asset_id = match (direct, path) {
        (Some(Some(a)), None) | (None, Some(Some(a))) => a,
        (Some(Some(a)), Some(Some(b))) if a == b => a,
        _ => return None,
    };
    Some(State::Created { asset_id })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{parse_multipart, Reply, TestServer};
    use rbxcloud::rbx::credentials::SecretString;

    fn fixture(root: &Path, creator_type: CreatorKind) -> BatchArgs {
        fs::write(root.join("first.png"), b"image bytes").unwrap();
        fs::write(root.join("second.png"), b"image bytes").unwrap();
        BatchArgs {
            paths: vec![root.to_string_lossy().into_owned()],
            creator_type,
            creator_id: 17,
            asset_type: AssetKind::Image,
            manifest: root.join("uploads.json"),
            concurrency: 2,
            dry_run: false,
        }
    }

    async fn run(args: &BatchArgs, server: &TestServer) -> Result<String> {
        let policy = ClientPolicy::loopback(server.addr())
            .unwrap()
            .max_retries(0);
        let transport = AsyncClient::new(
            Credentials::api_key(SecretString::new("fake-personal-key")),
            policy,
        )
        .await?;
        args.upload(
            args.prepare(None)?,
            AssetCreateClient::new(transport),
            Duration::ZERO,
        )
        .await
    }

    #[tokio::test]
    async fn resumes_accepted_batch_by_get_only_and_deduplicates_content() {
        let temp = tempfile::tempdir().unwrap();
        let group = fixture(temp.path(), CreatorKind::Group);
        let server = TestServer::start(vec![
            Reply::json(200, r#"{"path":"operations/id-1"}"#),
            Reply::empty(403),
            Reply::json(
                200,
                r#"{"operationId":"id-1","done":true,"response":{"assetId":"123","path":"assets/123"}}"#,
            ),
            Reply::json(200, r#"{"done":true,"response":{"assetId":"456"}}"#),
        ]);
        assert!(run(&group, &server).await.is_err());
        let result: serde_json::Value =
            serde_json::from_str(&run(&group, &server).await.unwrap()).unwrap();
        assert_eq!(result["entries"][0]["result"]["asset_id"], 123);
        assert_eq!(result["entries"][1]["result"]["asset_id"], 123);
        assert!(run(&group, &server).await.is_ok());
        let user = fixture(temp.path(), CreatorKind::User);
        assert!(run(&user, &server).await.is_ok());

        let requests = server.requests();
        assert_eq!(
            requests
                .iter()
                .map(|r| (r.method.as_str(), r.target.as_str()))
                .collect::<Vec<_>>(),
            [
                ("POST", "/assets/v1/assets"),
                ("GET", "/assets/v1/operations/id-1"),
                ("GET", "/assets/v1/operations/id-1"),
                ("POST", "/assets/v1/assets"),
            ]
        );
        for (i, creator) in [(0, "groupId"), (3, "userId")] {
            let r = &requests[i];
            assert_eq!(r.header("x-api-key"), Some("fake-personal-key"));
            let parts = parse_multipart(r.header("content-type").unwrap(), &r.body);
            assert_eq!(parts[0].name, "request");
            assert_eq!(parts[0].content_type.as_deref(), Some("application/json"));
            let metadata: serde_json::Value = serde_json::from_slice(&parts[0].data).unwrap();
            assert_eq!(metadata["assetType"], "Image");
            assert_eq!(
                metadata["creationContext"]["creator"],
                serde_json::json!({ (creator): "17" })
            );
            assert_eq!(parts[1].name, "fileContent");
            assert_eq!(parts[1].file_name.as_deref(), Some("first.png"));
            assert_eq!(parts[1].content_type.as_deref(), Some("image/png"));
            assert_eq!(parts[1].data, b"image bytes");
        }
    }

    #[tokio::test]
    async fn drop_after_post_never_replays_even_on_resume() {
        let temp = tempfile::tempdir().unwrap();
        let args = fixture(temp.path(), CreatorKind::Group);
        let server = TestServer::start(vec![Reply::close()]);
        assert!(run(&args, &server).await.is_err());
        assert!(run(&args, &server).await.is_err());
        let data = fs::read_to_string(&args.manifest).unwrap();
        assert!(data.contains("unconfirmed"));
        assert!(!data.contains("fake-personal-key"));
        assert_eq!(server.requests().len(), 1);
        assert_eq!(server.requests()[0].method, "POST");
    }

    #[tokio::test]
    async fn locked_manifest_blocks_create_before_any_requests() {
        let temp = tempfile::tempdir().unwrap();
        let args = fixture(temp.path(), CreatorKind::Group);
        let server = TestServer::start(vec![]);
        let _held = LockedManifest::open(&args.manifest).unwrap();
        let err = run(&args, &server).await.unwrap_err();
        assert!(err
            .to_string()
            .contains("cannot acquire batch manifest lock"));
        assert!(server.requests().is_empty());
    }
}
