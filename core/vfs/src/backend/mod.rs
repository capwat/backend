use std::any::Any;
use std::fmt::Debug;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::private::Sealed;
use crate::{File, Metadata, OpenOptions, Permissions, ReadDir, SetFileTimes};

mod imfs;
pub use self::imfs::InMemoryFs;

// Very self-explanatory reason of why we use path attribute to import a
// specific module instead of literally naming it in mod statement.
#[path = "std.rs"]
mod std_backend;
pub use self::std_backend::StdBackend;

/// This trait allows for the [`virtual file system handler`](super::Vfs) to interface
/// with this trait to communicate to various [file system backends].
#[allow(private_bounds)]
pub trait VfsBackend: Any + Debug + VfsInternalBackend + Sealed + Send + Sync + 'static {
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf>;

    fn current_dir(&self) -> io::Result<PathBuf>;
    fn set_current_dir(&self, path: &Path) -> io::Result<()>;

    fn read_dir(&self, path: &Path) -> io::Result<ReadDir>;
    fn join_path(&self, base: &Path, path: &Path) -> PathBuf;
    fn metadata(&self, path: &Path) -> io::Result<Metadata>;
}

/// Restricted trait for internal access with our APIs stuff. :)
pub(crate) trait VfsInternalBackend {
    fn open_file(&self, path: &Path, options: &mut OpenOptions) -> io::Result<File>;
}

/// This trait interfaces with the actual metadata implemented from
/// the backend chosen by the user to the code.
///
/// This trait is what makes implementing [`Metadata`](crate::Metadata) possible.
pub(crate) trait VfsMetadata: Debug {
    fn is_dir(&self) -> bool;
    fn is_file(&self) -> bool;
    fn is_symlink(&self) -> bool;

    fn permissions(&self) -> Permissions;
    fn size(&self) -> u64;

    fn modified(&self) -> io::Result<SystemTime>;
    fn accessed(&self) -> io::Result<SystemTime>;
    fn created(&self) -> io::Result<SystemTime>;
}

/// This trait interfaces with the actual file implemented from
/// the backend chosen by the user to the code.
///
/// This trait is what makes implementing [`File`](crate::File) possible.
pub(crate) trait VfsFile: Debug {
    fn io_flush(&mut self) -> io::Result<()>;
    fn io_read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
    fn io_seek(&mut self, pos: io::SeekFrom) -> io::Result<u64>;
    fn io_write(&mut self, buf: &[u8]) -> io::Result<usize>;

    fn metadata(&self) -> io::Result<Metadata>;
    fn set_len(&self, size: u64) -> io::Result<()>;
    fn set_times(&self, times: SetFileTimes) -> io::Result<()>;
    fn sync_all(&self) -> io::Result<()>;
    fn sync_data(&self) -> io::Result<()>;
}
