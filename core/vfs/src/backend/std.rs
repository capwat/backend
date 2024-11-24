//! [`std`]'s implementation of [`VfsBackend`]

use super::{VfsBackend, VfsFile, VfsInternalBackend, VfsMetadata};
use crate::private::Sealed;

use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct StdBackend(());

impl StdBackend {
    #[must_use]
    pub fn new() -> Self {
        Self(())
    }
}

impl VfsBackend for StdBackend {
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        fs_err::canonicalize(path)
    }

    fn current_dir(&self) -> io::Result<PathBuf> {
        std::env::current_dir()
    }

    fn set_current_dir(&self, path: &Path) -> io::Result<()> {
        std::env::set_current_dir(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<crate::ReadDir> {
        let inner = fs_err::read_dir(path)?;
        let inner = inner.map(|v| v.map(|entry| entry.path()));
        Ok(crate::ReadDir::new(inner))
    }

    fn join_path(&self, base: &Path, path: &Path) -> PathBuf {
        base.join(path)
    }

    fn metadata(&self, path: &Path) -> io::Result<crate::Metadata> {
        let metadata = fs_err::metadata(path)?;
        Ok(crate::Metadata::new(metadata))
    }
}

impl VfsInternalBackend for StdBackend {
    fn open_file(&self, path: &Path, options: &mut crate::OpenOptions) -> io::Result<crate::File> {
        let options: fs_err::OpenOptions = options.clone().into();
        let file = options.open(path)?;
        Ok(crate::File::new(file))
    }
}

impl Sealed for StdBackend {}

impl From<crate::OpenOptions> for fs_err::OpenOptions {
    fn from(value: crate::OpenOptions) -> Self {
        fs_err::OpenOptions::new()
            .create(value.create)
            .append(value.append)
            .truncate(value.truncate)
            .write(value.write)
            .create_new(value.create_new)
            .read(value.read)
            .clone()
    }
}

impl VfsFile for fs_err::File {
    fn io_flush(&mut self) -> io::Result<()> {
        std::io::Write::flush(self)
    }

    fn io_read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        std::io::Read::read(self, buf)
    }

    fn io_seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        std::io::Seek::seek(self, pos)
    }

    fn io_write(&mut self, buf: &[u8]) -> io::Result<usize> {
        std::io::Write::write(self, buf)
    }

    fn metadata(&self) -> io::Result<crate::Metadata> {
        fs_err::File::metadata(self).map(crate::Metadata::new)
    }

    fn set_len(&self, size: u64) -> io::Result<()> {
        fs_err::File::set_len(self, size)
    }

    #[cfg(not(windows))]
    fn set_times(&self, times: crate::SetFileTimes) -> io::Result<()> {
        if let Some(accessed) = times.accessed {
            filetime::set_file_atime(self.path(), accessed.into())?;
        }

        if let Some(modified) = times.modified.or(times.created) {
            filetime::set_file_mtime(self.path(), modified.into())?;
        }

        Ok(())
    }

    // filetime cannot set creation times on Windows, so implementing
    // this requires OS internal unsafe code.
    #[cfg(windows)]
    fn set_times(&self, times: crate::SetFileTimes) -> io::Result<()> {
        use filetime::FileTime;
        use std::fs::OpenOptions;
        use std::os::windows::prelude::{AsRawHandle, OpenOptionsExt};
        use windows_sys::Win32::Foundation::{FILETIME, HANDLE};
        use windows_sys::Win32::Storage::FileSystem::*;

        let path = self.path();
        let file = OpenOptions::new()
            .write(true)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
            .open(path)?;

        let metadata = fs_err::metadata(path)?;

        let accessed = times
            .accessed
            .map(FileTime::from_system_time)
            .unwrap_or_else(|| FileTime::from_last_access_time(&metadata));

        let created = times
            .created
            .map(FileTime::from_system_time)
            .or_else(|| FileTime::from_creation_time(&metadata));

        let modified = times
            .modified
            .map(FileTime::from_system_time)
            .or_else(|| metadata.modified().ok().map(FileTime::from_system_time));

        let accessed = to_win_filetime(accessed);
        let created = created
            .zip(modified)
            .map(|(created, modified)| {
                if created.unix_seconds() < modified.unix_seconds() {
                    created
                } else {
                    modified
                }
            })
            .map(to_win_filetime);

        let modified = modified.map(to_win_filetime);

        fn to_win_filetime(ft: filetime::FileTime) -> FILETIME {
            let intervals =
                ft.seconds() * (1_000_000_000 / 100) + ((ft.nanoseconds() as i64) / 100);

            FILETIME {
                dwLowDateTime: intervals as u32,
                dwHighDateTime: (intervals >> 32) as u32,
            }
        }

        return unsafe {
            let ret = SetFileTime(
                file.as_raw_handle() as HANDLE,
                created
                    .as_ref()
                    .map(|p| p as *const FILETIME)
                    .unwrap_or(std::ptr::null()),
                (&accessed) as *const FILETIME,
                modified
                    .as_ref()
                    .map(|p| p as *const FILETIME)
                    .unwrap_or(std::ptr::null()),
            );
            if ret != 0 {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error().into())
            }
        };
    }

    fn sync_all(&self) -> io::Result<()> {
        todo!()
    }

    fn sync_data(&self) -> io::Result<()> {
        todo!()
    }
}

impl VfsMetadata for std::fs::Metadata {
    fn is_dir(&self) -> bool {
        std::fs::Metadata::is_dir(self)
    }

    fn is_file(&self) -> bool {
        std::fs::Metadata::is_file(self)
    }

    fn is_symlink(&self) -> bool {
        std::fs::Metadata::is_symlink(self)
    }

    fn permissions(&self) -> crate::Permissions {
        // Assume that all real files are writable if it is not readonly.
        let perms = std::fs::Metadata::permissions(self);
        if perms.readonly() {
            crate::Permissions::readonly()
        } else {
            crate::Permissions::writable()
        }
    }

    fn size(&self) -> u64 {
        self.len()
    }

    fn modified(&self) -> io::Result<std::time::SystemTime> {
        std::fs::Metadata::modified(self)
    }

    fn accessed(&self) -> io::Result<std::time::SystemTime> {
        std::fs::Metadata::accessed(self)
    }

    fn created(&self) -> io::Result<std::time::SystemTime> {
        std::fs::Metadata::created(self)
    }
}
