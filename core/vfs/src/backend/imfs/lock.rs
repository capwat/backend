use super::utils::{ImfsPath, ImfsPathBuf};
use super::InMemoryFsImpl;

use dashmap::DashMap;
use parking_lot::{ArcRwLockReadGuard, ArcRwLockWriteGuard, RawRwLock, RwLock};
use std::fmt::Debug;
use std::sync::Arc;
// use tracing::{debug, warn};

impl InMemoryFsImpl {
    // #[tracing::instrument(skip(self))]
    pub fn lock_path_for_read(&self, path: &ImfsPath) -> ImfsPathReadLockGuard {
        while let Some(locked_descendant) = self.get_locked_descendant(path) {
            let Some(entry) = self.locks.get(&locked_descendant) else {
                continue;
            };

            let _ = entry.value().read_arc();
        }

        let lock = Arc::new(RwLock::new(()));
        self.locks.insert(path.to_path_buf(), lock.clone());

        ImfsPathReadLockGuard {
            inner: Some(lock.read_arc()),
            path: path.to_path_buf(),
            handle: self.locks.clone(),
        }
    }

    pub fn lock_path_for_write(&self, path: &ImfsPath) -> ImfsPathWriteLockGuard {
        while let Some(locked_descendant) = self.get_locked_descendant(path) {
            let Some(entry) = self.locks.get(&locked_descendant) else {
                continue;
            };

            let _ = entry.value().read_arc();
        }

        let lock = Arc::new(RwLock::new(()));
        self.locks.insert(path.to_path_buf(), lock.clone());

        ImfsPathWriteLockGuard {
            inner: Some(lock.write_arc()),
            path: path.to_path_buf(),
            handle: self.locks.clone(),
        }
    }

    fn get_locked_descendant(&self, path: &ImfsPath) -> Option<ImfsPathBuf> {
        let mut maybe_parent_path = Some(path.to_path_buf());
        while let Some(path) = maybe_parent_path.as_ref() {
            let Some(entry) = self.locks.get(path) else {
                maybe_parent_path = path.parent().map(|v| v.to_path_buf());
                continue;
            };

            let entry = entry.value();
            let is_locked = entry.try_read_arc().is_none();

            if is_locked {
                return Some(path.to_path_buf());
            }
            maybe_parent_path = path.parent().map(|v| v.to_path_buf());
        }
        None
    }
}

pub struct ImfsPathReadLockGuard {
    handle: Arc<DashMap<ImfsPathBuf, Arc<RwLock<()>>>>,
    inner: Option<ArcRwLockReadGuard<RawRwLock, ()>>,
    path: ImfsPathBuf,
}

impl Debug for ImfsPathReadLockGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImfsPathReadLockGuard")
            .finish_non_exhaustive()
    }
}

impl Drop for ImfsPathReadLockGuard {
    fn drop(&mut self) {
        self.inner.take();
        self.handle.remove(&self.path);
    }
}

pub struct ImfsPathWriteLockGuard {
    handle: Arc<DashMap<ImfsPathBuf, Arc<RwLock<()>>>>,
    inner: Option<ArcRwLockWriteGuard<RawRwLock, ()>>,
    path: ImfsPathBuf,
}

impl Debug for ImfsPathWriteLockGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImfsPathWriteLockGuard")
            .finish_non_exhaustive()
    }
}

impl Drop for ImfsPathWriteLockGuard {
    fn drop(&mut self) {
        self.inner.take();
        self.handle.remove(&self.path);
    }
}
