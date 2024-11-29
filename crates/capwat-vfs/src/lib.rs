mod private;
mod snapshot;

pub mod backend;
pub use self::backend::VfsBackend;
pub use self::snapshot::VfsSnapshot;

use self::backend::{VfsFile, VfsMetadata};
use bitflags::bitflags;
use std::any::TypeId;
use std::fmt::Debug;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct Vfs(Arc<dyn VfsBackend>);

impl Vfs {
    #[must_use]
    pub fn new(backend: impl VfsBackend) -> Self {
        Self(Arc::new(backend))
    }

    #[must_use]
    pub fn new_std() -> Self {
        Self(Arc::new(self::backend::StdBackend::new()))
    }

    pub fn current_dir(&self) -> io::Result<PathBuf> {
        self.0.current_dir()
    }

    pub fn canonicalize<P: AsRef<Path>>(&self, path: P) -> io::Result<PathBuf> {
        self.0.canonicalize(path.as_ref())
    }

    #[must_use]
    pub fn exists<P: AsRef<Path>>(&self, path: P) -> bool {
        self.metadata(path).is_ok()
    }

    #[must_use]
    pub fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool {
        self.metadata(path).map(|v| v.is_dir()).unwrap_or_default()
    }

    #[must_use]
    pub fn is_file<P: AsRef<Path>>(&self, path: P) -> bool {
        self.metadata(path).map(|v| v.is_file()).unwrap_or_default()
    }

    #[must_use]
    pub fn is_using_std_backend(&self) -> bool {
        self.0.type_id() == TypeId::of::<self::backend::StdBackend>()
    }

    pub fn metadata<P: AsRef<Path>>(&self, path: P) -> io::Result<Metadata> {
        self.0.metadata(path.as_ref())
    }

    pub fn join_path<P: AsRef<Path>, M: AsRef<Path>>(&self, base: P, path: M) -> PathBuf {
        self.0.join_path(base.as_ref(), path.as_ref())
    }

    pub fn read_dir<P: AsRef<Path>>(&self, path: P) -> io::Result<ReadDir> {
        self.0.read_dir(path.as_ref())
    }

    pub fn read<P: AsRef<Path>>(&self, path: P) -> io::Result<Vec<u8>> {
        let mut file = OpenOptions::new().read(true).open(self, path)?;
        let mut bytes = Vec::new();
        let size = file.metadata().map(|v| v.size() as usize).ok();
        bytes.try_reserve_exact(size.unwrap_or(0))?;
        file.read_to_end(&mut bytes)?;

        Ok(bytes)
    }

    pub fn read_to_string<P: AsRef<Path>>(&self, path: P) -> io::Result<String> {
        let mut file = OpenOptions::new().read(true).open(self, path)?;
        let size = file.metadata().map(|v| v.size() as usize).ok();

        let mut string = String::new();
        string.try_reserve_exact(size.unwrap_or(0))?;
        file.read_to_string(&mut string)?;

        Ok(string)
    }

    pub fn set_current_dir<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self.0.set_current_dir(path.as_ref())
    }

    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&self, path: P, contents: C) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(self, path)?;

        file.write_all(contents.as_ref())
    }
}

bitflags! {
    /// Representation of the various permissions on a file.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Permissions: u8 {
        const CAN_READ = 1 << 1;
        const CAN_WRITE = 1 << 2;
    }
}

impl Default for Permissions {
    fn default() -> Self {
        Self::writable()
    }
}

impl Permissions {
    #[must_use]
    pub fn readonly() -> Self {
        Self::CAN_READ
    }

    #[must_use]
    pub fn writable() -> Self {
        Self::CAN_READ | Self::CAN_WRITE
    }

    pub fn can_read(&self) -> bool {
        self.intersects(Self::CAN_READ)
    }

    pub fn can_write(&self) -> bool {
        self.intersects(Self::CAN_WRITE)
    }

    pub fn is_readonly(&self) -> bool {
        self.intersects(Self::CAN_READ) && !self.intersects(Self::CAN_WRITE)
    }

    pub fn set_readonly(&mut self, readonly: bool) {
        if readonly {
            *self = Self::CAN_READ;
        } else {
            *self = Self::CAN_READ | Self::CAN_WRITE;
        }
    }
}

/// An object providing access to an open file on the file system.
///
/// This is similar to [`std::fs::File`] but unlike the [`std`] counterpart, this
/// struct interfaces with its internal file object from its assigned [virtual file system]
/// set by the user.
///
/// # Platform-specific behavior
///
/// - **For [`std`] backend**: Please read the documentation of [`std::fs::File`] object to learn about its behaviors.
///
/// [virtual file system]: VfsBackend
pub struct File(Box<dyn VfsFile>);

impl Debug for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("VfsFile").field(&self.0).finish()
    }
}

impl File {
    #[inline(always)]
    #[must_use]
    pub(crate) fn new(file: impl VfsFile + 'static) -> Self {
        Self(Box::new(file))
    }

    pub fn metadata(&self) -> io::Result<Metadata> {
        self.0.metadata()
    }

    pub fn set_len(&self, size: u64) -> io::Result<()> {
        self.0.set_len(size)
    }

    pub fn set_times(&self, times: SetFileTimes) -> io::Result<()> {
        self.0.set_times(times)
    }

    pub fn sync_all(&self) -> io::Result<()> {
        self.0.sync_all()
    }

    pub fn sync_data(&self) -> io::Result<()> {
        self.0.sync_data()
    }
}

impl std::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        VfsFile::io_read(&mut *self.0, buf)
    }
}

impl std::io::Seek for File {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        VfsFile::io_seek(&mut *self.0, pos)
    }
}

impl std::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        VfsFile::io_write(&mut *self.0, buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        VfsFile::io_flush(&mut *self.0)
    }
}

/// Holds parameters to set file times.
///
/// All parameters by default are not set unless called by the user
/// with setter functions like [`set_accessed`](SetFileTimes::set_accessed) and etc.
#[derive(Debug, Default)]
#[must_use]
pub struct SetFileTimes {
    pub accessed: Option<SystemTime>,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
}

impl SetFileTimes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_accessed(self, time: SystemTime) -> Self {
        Self {
            accessed: Some(time),
            ..self
        }
    }

    pub fn set_created(self, time: SystemTime) -> Self {
        Self {
            created: Some(time),
            ..self
        }
    }

    pub fn set_modified(self, time: SystemTime) -> Self {
        Self {
            modified: Some(time),
            ..self
        }
    }
}

/// Metadata information about a file.
///
/// This is similar to [`std::fs::Metadata`] but this struct interfaces with
/// the internal metadata from its [virtual file system backend](VfsBackend).
pub struct Metadata(Box<dyn VfsMetadata>);

impl Debug for Metadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("VfsMetadata").field(&self.0).finish()
    }
}

impl Metadata {
    #[inline(always)]
    #[must_use]
    pub(crate) fn new(metadata: impl VfsMetadata + 'static) -> Self {
        Self(Box::new(metadata))
    }

    #[must_use]
    pub fn is_dir(&self) -> bool {
        self.0.is_dir()
    }

    #[must_use]
    pub fn is_file(&self) -> bool {
        self.0.is_file()
    }

    #[must_use]
    pub fn is_symlink(&self) -> bool {
        self.0.is_symlink()
    }

    #[must_use]
    pub fn permissions(&self) -> Permissions {
        self.0.permissions()
    }

    #[must_use]
    pub fn size(&self) -> u64 {
        self.0.size()
    }

    pub fn modified(&self) -> io::Result<SystemTime> {
        self.0.modified()
    }

    pub fn accessed(&self) -> io::Result<SystemTime> {
        self.0.accessed()
    }

    pub fn created(&self) -> io::Result<SystemTime> {
        self.0.created()
    }
}

/// Iterator over the entries in a directory.
///
/// This is like [`std::fs::ReadDir`] but this struct interfaces with the internal
/// metadata from its [virtual file system backend](VfsBackend).
///
/// Unlike [`std::fs::ReadDir`] where it has helper functions to conveniently
/// access file system operations like [`::metadata`] from its directory entry,
/// this struct doesn't have any convenient functions and it simply iterates the
/// entries' path from the directory.
///
/// [`::metadata`]: std::fs::DirEntry
pub struct ReadDir(Box<dyn Iterator<Item = io::Result<PathBuf>>>);

impl ReadDir {
    #[inline(always)]
    #[must_use]
    pub(crate) fn new(inner: impl Iterator<Item = io::Result<PathBuf>> + 'static) -> Self {
        Self(Box::new(inner))
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Options and flags which can be used to configure how a file is opened.
#[derive(Debug, Clone)]
#[must_use]
pub struct OpenOptions {
    pub(crate) read: bool,
    pub(crate) write: bool,
    pub(crate) append: bool,
    pub(crate) truncate: bool,
    pub(crate) create: bool,
    pub(crate) create_new: bool,
}

impl OpenOptions {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }

    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }

    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }

    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }

    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }

    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }

    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    pub fn open<P: AsRef<Path>>(&mut self, vfs: &Vfs, path: P) -> io::Result<File> {
        vfs.0.open_file(path.as_ref(), self)
    }
}

pub trait VfsResultExt {
    type Ok;

    fn optional(self) -> io::Result<Option<Self::Ok>>;
}

impl<Ok> VfsResultExt for io::Result<Ok> {
    type Ok = Ok;

    fn optional(self) -> io::Result<Option<Ok>> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Vfs;
    use static_assertions::assert_impl_all;

    assert_impl_all!(Vfs: Send, Sync);
}
