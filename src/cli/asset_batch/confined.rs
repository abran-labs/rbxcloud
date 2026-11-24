use std::{fs::File, io, path::Path};

use anyhow::Result;

use super::{Children, Kind};

#[cfg(target_os = "linux")]
pub(super) use linux::{Project, StateDir};

#[cfg(target_os = "linux")]
mod linux {
    use std::{
        io::Write,
        os::unix::{ffi::OsStrExt, fs::MetadataExt},
        path::Component,
    };

    use anyhow::{ensure, Context};
    use rustix::fs::{self, AtFlags, Dir, FileType, Mode, OFlags, ResolveFlags, CWD};

    use super::*;

    const STATE_DIR: &str = ".rbxcloud";
    const MANIFEST: &str = "upload-batch.json";
    const LOCK: &str = "upload-batch.json.lock";

    pub(crate) struct Project(File);
    pub(crate) struct StateDir(File);

    impl Project {
        pub(crate) fn open(path: &Path) -> Result<Self> {
            ensure!(path.is_absolute(), "project root must be absolute");
            let dir = fs::openat2(
                CWD,
                path,
                OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
                Mode::empty(),
                ResolveFlags::NO_SYMLINKS,
            )
            .context("workflow uploads require Linux openat2 and a symlink-free project root")?;
            Ok(Self(File::from(dir)))
        }

        fn open_relative(&self, path: &Path, flags: OFlags) -> io::Result<File> {
            if !path
                .components()
                .all(|part| matches!(part, Component::Normal(_) | Component::CurDir))
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "not a project-relative path",
                ));
            }
            // Resolve the whole path from the held root in one kernel operation.
            // Never retry with an unconfined open (including on older kernels).
            fs::openat2(
                &self.0,
                path,
                flags | OFlags::CLOEXEC,
                Mode::empty(),
                ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_XDEV,
            )
            .map(File::from)
            .map_err(Into::into)
        }

        pub(crate) fn kind(&self, path: &Path) -> io::Result<Kind> {
            let meta = self.open_relative(path, OFlags::PATH)?.metadata()?;
            Ok(Kind::from_std(meta.file_type()))
        }

        pub(crate) fn children(&self, path: &Path) -> Result<Children> {
            let directory = self.open_relative(path, OFlags::RDONLY | OFlags::DIRECTORY)?;
            let path = path.to_owned();
            let iter = Dir::read_from(&directory)?
                .map(move |entry| -> Result<Option<_>> {
                    let entry = entry?;
                    let name = entry.file_name().to_bytes();
                    if matches!(name, b"." | b"..") {
                        return Ok(None);
                    }
                    let name = std::ffi::OsStr::from_bytes(name);
                    let ty = match entry.file_type() {
                        FileType::Unknown => FileType::from_raw_mode(
                            fs::statat(&directory, name, AtFlags::SYMLINK_NOFOLLOW)?.st_mode,
                        ),
                        ty => ty,
                    };
                    let kind = match ty {
                        FileType::Directory => Kind::Directory,
                        FileType::RegularFile => Kind::File,
                        _ => Kind::Other,
                    };
                    Ok(Some((path.join(name), kind)))
                })
                .filter_map(Result::transpose);
            Ok(Box::new(iter))
        }

        pub(crate) fn file(&self, path: &Path) -> Result<File> {
            let file = self.open_relative(path, OFlags::RDONLY | OFlags::NONBLOCK)?;
            regular(&file)?;
            Ok(file)
        }

        pub(crate) fn state_dir(&self) -> Result<StateDir> {
            let path = Path::new(STATE_DIR);
            let flags = OFlags::RDONLY | OFlags::DIRECTORY;
            let dir = match self.open_relative(path, flags) {
                Ok(dir) => dir,
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    match fs::mkdirat(&self.0, path, Mode::RWXU) {
                        Ok(()) | Err(rustix::io::Errno::EXIST) => {}
                        Err(e) => return Err(e.into()),
                    }
                    self.open_relative(path, flags)?
                }
                Err(e) => return Err(e.into()),
            };
            let meta = dir.metadata()?;
            ensure!(
                meta.uid() == rustix::process::geteuid().as_raw() && meta.mode() & 0o022 == 0,
                "project .rbxcloud must be owned by the current user and not writable by group or others"
            );
            // Persist the state directory entry, even if init just created it.
            self.open_relative(Path::new("."), flags)?.sync_all()?;
            Ok(StateDir(dir))
        }
    }

    fn regular(file: &File) -> Result<()> {
        let meta = file.metadata()?;
        ensure!(
            meta.is_file() && meta.nlink() == 1,
            "workflow input and state must be regular files without hard links"
        );
        Ok(())
    }

    impl StateDir {
        fn open(&self, name: &str, flags: OFlags) -> io::Result<File> {
            // Names are single components; every operation uses the same pinned dir.
            fs::openat2(
                &self.0,
                name,
                flags | OFlags::CLOEXEC | OFlags::NONBLOCK,
                if flags.contains(OFlags::CREATE) {
                    Mode::RUSR | Mode::WUSR
                } else {
                    Mode::empty()
                },
                ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_XDEV,
            )
            .map(File::from)
            .map_err(Into::into)
        }

        pub(crate) fn lock(&self) -> Result<File> {
            let file = self.open(LOCK, OFlags::RDWR | OFlags::CREATE)?;
            regular(&file)?;
            Ok(file)
        }

        pub(crate) fn manifest(&self) -> Result<Option<File>> {
            match self.open(MANIFEST, OFlags::RDONLY) {
                Ok(file) => {
                    regular(&file)?;
                    Ok(Some(file))
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(e.into()),
            }
        }

        pub(crate) fn replace(&self, bytes: &[u8]) -> Result<()> {
            let name = format!(".upload-batch-{:032x}.tmp", rand::random::<u128>());
            let mut temp = self.open(&name, OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL)?;
            let result = (|| -> Result<()> {
                temp.write_all(bytes)?;
                temp.sync_all()?;
                fs::renameat(&self.0, name.as_str(), &self.0, MANIFEST)?;
                Ok(())
            })();
            if result.is_err() {
                let _ = fs::unlinkat(&self.0, name.as_str(), AtFlags::empty());
            }
            result.context("cannot replace batch manifest")?;
            self.0.sync_all()?;
            Ok(())
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub(super) use unavailable::{Project, StateDir};

#[cfg(not(target_os = "linux"))]
mod unavailable {
    use super::*;

    pub(crate) struct Project(std::convert::Infallible);
    pub(crate) struct StateDir(std::convert::Infallible);

    impl Project {
        pub(crate) fn open(_: &Path) -> Result<Self> {
            anyhow::bail!("workflow uploads require Linux openat2 for project path confinement")
        }
        pub(crate) fn kind(&self, _: &Path) -> io::Result<Kind> {
            match self.0 {}
        }
        pub(crate) fn children(&self, _: &Path) -> Result<Children> {
            match self.0 {}
        }
        pub(crate) fn file(&self, _: &Path) -> Result<File> {
            match self.0 {}
        }
        pub(crate) fn state_dir(&self) -> Result<StateDir> {
            match self.0 {}
        }
    }

    impl StateDir {
        pub(crate) fn lock(&self) -> Result<File> {
            match self.0 {}
        }
        pub(crate) fn manifest(&self) -> Result<Option<File>> {
            match self.0 {}
        }
        pub(crate) fn replace(&self, _: &[u8]) -> Result<()> {
            match self.0 {}
        }
    }
}
