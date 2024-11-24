use super::{ImfsEntry, ImfsEntryTimes};
use crate::backend::VfsMetadata;
use crate::Permissions;

use std::io;
use std::time::SystemTime;

#[derive(Debug)]
pub struct ImfsMetadataSnapshot {
    is_dir: bool,
    permissions: Permissions,
    size: u64,
    times: ImfsEntryTimes,
}

impl ImfsMetadataSnapshot {
    #[must_use]
    pub fn from_entry(entry: &ImfsEntry, size: u64) -> Self {
        match entry {
            ImfsEntry::Directory {
                permissions, times, ..
            } => ImfsMetadataSnapshot {
                is_dir: true,
                permissions: *permissions,
                size,
                times: *times,
            },
            ImfsEntry::File {
                permissions, times, ..
            } => ImfsMetadataSnapshot {
                is_dir: false,
                permissions: *permissions,
                size,
                times: *times,
            },
        }
    }
}

impl VfsMetadata for ImfsMetadataSnapshot {
    fn is_dir(&self) -> bool {
        self.is_dir
    }

    fn is_file(&self) -> bool {
        !self.is_dir
    }

    fn is_symlink(&self) -> bool {
        false
    }

    fn permissions(&self) -> Permissions {
        self.permissions
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn modified(&self) -> io::Result<SystemTime> {
        Ok(self.times.modified)
    }

    fn accessed(&self) -> io::Result<SystemTime> {
        Ok(self.times.accessed)
    }

    fn created(&self) -> io::Result<SystemTime> {
        Ok(self.times.created)
    }
}
