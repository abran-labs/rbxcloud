use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{bail, ensure, Context, Result};
use clap::{Args, Subcommand};

const NAME: &str = "roblox-open-cloud";
const SKILL: &str = include_str!("../../skills/roblox-open-cloud/SKILL.md");

#[derive(Debug, Args)]
pub struct Skill {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Install the matching skill (does not silently overwrite local edits)
    Install {
        #[command(flatten)]
        destination: Destination,
        #[arg(long, conflicts_with = "print")]
        force: bool,
        /// Print the skill to stdout instead of writing a file
        #[arg(long, conflicts_with_all = ["claude", "opencode", "dir"])]
        print: bool,
    },
    /// Inspect installed skill without writing
    Status(Destination),
}

#[derive(Debug, Args)]
struct Destination {
    /// Use ~/.claude/skills
    #[arg(long, conflicts_with_all = ["opencode", "dir"])]
    claude: bool,
    /// Use $XDG_CONFIG_HOME/opencode/skills (or ~/.config/opencode/skills)
    #[arg(long, conflicts_with = "dir")]
    opencode: bool,
    /// Use a custom skills root
    #[arg(long)]
    dir: Option<PathBuf>,
}

pub(super) fn home() -> Result<PathBuf> {
    env::var_os("HOME")
        .filter(|s| !s.is_empty())
        .or_else(|| env::var_os("USERPROFILE").filter(|s| !s.is_empty()))
        .map(PathBuf::from)
        .context("cannot locate home directory")
}

pub(super) fn config_home() -> Result<PathBuf> {
    match env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
    {
        Some(path) => Ok(path),
        None => Ok(home()?.join(".config")),
    }
}

impl Destination {
    fn directory(&self) -> Result<PathBuf> {
        let root = if let Some(path) = &self.dir {
            path.clone()
        } else if self.claude {
            home()?.join(".claude/skills")
        } else if self.opencode {
            config_home()?.join("opencode/skills")
        } else {
            home()?.join(".agents/skills")
        };
        Ok(root.join(NAME))
    }
}

fn inspect(dir: &Path, expected: &[u8]) -> Result<Option<bool>> {
    match dir.symlink_metadata() {
        Ok(meta) => ensure!(
            meta.is_dir(),
            "skill directory must not be a symlink or special file"
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let path = dir.join("SKILL.md");
    match path.symlink_metadata() {
        Ok(meta) => {
            ensure!(
                meta.is_file(),
                "SKILL.md must not be a symlink or special file"
            );
            Ok(Some(
                meta.len() == expected.len() as u64 && fs::read(path)? == expected,
            ))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

impl Skill {
    pub fn run(self) -> Result<Option<String>> {
        let content = SKILL.replace("@RBXCLOUD_VERSION@", env!("CARGO_PKG_VERSION"));
        let message = match self.command {
            Command::Install { print: true, .. } => content.trim_end_matches('\n').into(),
            Command::Status(destination) => {
                let dir = destination.directory()?;
                let state = match inspect(&dir, content.as_bytes())? {
                    None => "not installed",
                    Some(true) => "matches this binary",
                    Some(false) => "differs from this binary",
                };
                format!("{}: {state}", dir.join("SKILL.md").display())
            }
            Command::Install {
                destination,
                print: false,
                force,
            } => {
                let dir = destination.directory()?;
                let path = dir.join("SKILL.md");
                let prior = inspect(&dir, content.as_bytes())?;
                if prior == Some(true) {
                    return Ok(Some(format!("Already current: {}", path.display())));
                }
                if prior == Some(false) && !force {
                    bail!("{} differs; review changes or use --force", path.display());
                }
                fs::create_dir_all(&dir)?;
                let mut temp = tempfile::NamedTempFile::new_in(&dir)?;
                temp.write_all(content.as_bytes())?;
                temp.as_file().sync_all()?;
                if force {
                    temp.persist(&path)?;
                } else {
                    temp.persist_noclobber(&path)
                        .context("SKILL.md appeared during install; not overwritten")?;
                }
                format!("Installed {}", path.display())
            }
        };
        Ok(Some(message))
    }
}
