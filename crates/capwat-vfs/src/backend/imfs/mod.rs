mod backend;
mod entry;
mod file;
mod lock;
mod metadata;
mod snapshot;
mod utils;

pub use self::entry::*;
pub use self::metadata::*;
pub use self::utils::*;

use dashmap::DashMap;
use parking_lot::RwLock;
use std::fmt::Debug;
use std::io;
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Clone)]
pub struct InMemoryFs {
    handle: Arc<InMemoryFsImpl>,
}

impl Default for InMemoryFs {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryFs {
    #[must_use]
    pub fn new() -> Self {
        Self {
            handle: Arc::new(InMemoryFsImpl::new()),
        }
    }

    pub fn set_current_dir<P: AsRef<ImfsPath>>(&self, path: P) -> io::Result<()> {
        self.handle.set_current_dir(path.as_ref())
    }
}

impl Debug for InMemoryFs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.handle, f)
    }
}

pub struct InMemoryFsImpl {
    current_dir: RwLock<Option<ImfsPathBuf>>,
    data: DashMap<ImfsPathBuf, Arc<Vec<u8>>>,
    entries: DashMap<ImfsPathBuf, ImfsEntry>,
    locks: Arc<DashMap<ImfsPathBuf, Arc<RwLock<()>>>>,
}

impl InMemoryFsImpl {
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_dir: RwLock::new(None),
            data: DashMap::new(),
            entries: DashMap::new(),
            locks: Arc::new(DashMap::new()),
        }
    }
}

impl InMemoryFsImpl {
    pub fn canonicalize(&self, path: &ImfsPath) -> io::Result<ImfsPathBuf> {
        #[cfg(not(windows))]
        use std::path::Component;
        #[cfg(windows)]
        use unix_path::Component;

        let mut realpath = self.current_dir()?;
        for component in path.components() {
            match component {
                // unix_path doesn't have Prefix variant in their Component enum
                #[cfg(not(windows))]
                Component::Prefix(..) => {}
                Component::RootDir => {
                    realpath = ImfsPathBuf::from("/");
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    // remain as it is if it hasn't found a parent
                    if let Some(parent) = realpath.parent() {
                        realpath = parent.to_path_buf();
                    };
                }
                Component::Normal(part) => {
                    realpath = realpath.join(part);
                }
            }
        }

        // making sure it does exists
        if self.entries.get(&realpath).is_none() {
            return not_found(&realpath);
        }

        Ok(realpath)
    }

    pub fn current_dir(&self) -> io::Result<ImfsPathBuf> {
        let guard = self.current_dir.read();
        let current_dir = guard.as_ref();
        let Some(path) = current_dir.as_ref().cloned() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "current directory is not set".to_string(),
            ));
        };

        // We need to also check if the current directory actually
        // exists and it is a directory
        let Some(entry) = self.entries.get(path) else {
            return utils::not_found(path);
        };

        match entry.value() {
            ImfsEntry::Directory { .. } => Ok(path.to_path_buf()),
            _ => utils::must_be_dir(path),
        }
    }

    pub fn read_dir(&self, path: &ImfsPath) -> io::Result<crate::ReadDir> {
        let realpath = self.realpath(path);
        let Some(mut entry) = self.entries.get_mut(&realpath) else {
            return not_found(&realpath);
        };

        let _guard = self.lock_path_for_read(&realpath);
        match entry.value_mut() {
            ImfsEntry::Directory {
                children, times, ..
            } => {
                times.accessed = SystemTime::now();

                let entries = children
                    .iter()
                    .cloned()
                    .map(|v| Ok(utils::to_std_path(&v)))
                    .collect::<Vec<_>>();

                Ok(crate::ReadDir::new(entries.into_iter()))
            }
            ImfsEntry::File { .. } => must_be_dir(path),
        }
    }

    pub fn set_current_dir(&self, path: &ImfsPath) -> io::Result<()> {
        // We need to also check if the new current directory
        // actually exists and it is a directory
        let realpath = self.realpath(path);
        let metadata = self.metadata(&realpath)?;
        if !metadata.is_dir() {
            return must_be_dir(path);
        }

        *self.current_dir.write() = Some(realpath);
        Ok(())
    }

    pub fn metadata(&self, path: &ImfsPath) -> io::Result<crate::Metadata> {
        let realpath = self.realpath(path);
        let Some(entry) = self.entries.get(&realpath) else {
            return not_found(&realpath);
        };

        let size = match entry.value() {
            ImfsEntry::Directory { .. } => 0,
            ImfsEntry::File { .. } => self
                .data
                .get(path)
                .expect("unexpected file got not data")
                .value()
                .len() as u64,
        };

        let metadata = ImfsMetadataSnapshot::from_entry(&entry, size);
        Ok(crate::Metadata::new(metadata))
    }

    pub fn realpath(&self, path: &ImfsPath) -> ImfsPathBuf {
        self.canonicalize(path)
            .unwrap_or_else(|_| path.to_path_buf())
    }
}

impl Debug for InMemoryFsImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryFs")
            .field("current_dir", &self.current_dir)
            .field("entries", &self.entries)
            .field("locks", &self.locks)
            .finish_non_exhaustive()
    }
}
