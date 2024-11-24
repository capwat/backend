use super::InMemoryFs;

use crate::backend::{VfsBackend, VfsInternalBackend};
use crate::private::Sealed;

use std::io;
use std::path::{Path, PathBuf};

impl VfsBackend for InMemoryFs {
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        let path = super::utils::to_unix_path(path);
        let unix_path = self.handle.canonicalize(&path)?;
        Ok(super::utils::to_std_path(&unix_path))
    }

    fn current_dir(&self) -> io::Result<PathBuf> {
        let path = self.handle.current_dir()?;
        Ok(super::utils::to_std_path(&path))
    }

    fn set_current_dir(&self, path: &Path) -> io::Result<()> {
        let path = super::utils::to_unix_path(path);
        self.handle.set_current_dir(&path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<crate::ReadDir> {
        let path = super::utils::to_unix_path(path);
        self.handle.read_dir(&path)
    }

    fn join_path(&self, base: &Path, path: &Path) -> PathBuf {
        let base = super::utils::to_unix_path(base);
        let path = super::utils::to_unix_path(path);
        super::utils::to_std_path(&base.join(path))
    }

    fn metadata(&self, path: &Path) -> io::Result<crate::Metadata> {
        let path = super::utils::to_unix_path(path);
        self.handle.metadata(&path)
    }
}

impl VfsInternalBackend for InMemoryFs {
    fn open_file(&self, path: &Path, options: &mut crate::OpenOptions) -> io::Result<crate::File> {
        let path = super::utils::to_unix_path(path);
        self.handle.clone().open_file(&path, options)
    }
}

impl Sealed for InMemoryFs {}
