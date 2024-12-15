use std::io::{self, Cursor, Seek, Write};
use std::sync::Arc;
use std::time::SystemTime;

use super::lock::ImfsPathReadLockGuard;
use super::{ImfsEntry, ImfsEntryTimes, ImfsPath, ImfsPathBuf, InMemoryFsImpl};
use crate::backend::VfsFile;
use crate::Permissions;

impl InMemoryFsImpl {
    pub fn assert_desecendants_exists(&self, path: &ImfsPath) -> io::Result<()> {
        // checking make sure that each descendants exists
        let mut parent = path.parent().map(|v| v.to_path_buf());
        while let Some(value) = parent {
            self.metadata(&value)?;
            parent = value.parent().map(|v| v.to_path_buf());
        }
        Ok(())
    }

    // #[tracing::instrument(skip(self))]
    pub fn create_dir(
        &self,
        path: &ImfsPath,
        perms: Option<Permissions>,
        ignore_if_exists: bool,
    ) -> io::Result<()> {
        let realpath = self.realpath(path);

        // if we're making a root directory, we don't need to.
        if realpath.components().count() != 1 {
            self.assert_desecendants_exists(&realpath)?;
        }

        if self.entries.get(&realpath).is_some() {
            return if ignore_if_exists {
                Ok(())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("{} already exists", realpath.display()),
                ))
            };
        }

        let _guard = self.lock_path_for_write(path);
        self.entries.insert(
            realpath.clone(),
            ImfsEntry::Directory {
                children: Vec::new(),
                permissions: perms.unwrap_or_default(),
                times: ImfsEntryTimes::now(),
            },
        );

        Ok(())
    }

    // #[tracing::instrument(skip(self, data), fields(
    //     data.len = %data.as_ref().map(|v| v.len()).unwrap_or(0)
    // ))]
    pub fn create_file(
        &self,
        path: &ImfsPath,
        data: Option<Vec<u8>>,
        perms: Option<Permissions>,
        ignore_if_exists: bool,
    ) -> io::Result<()> {
        let realpath = self.realpath(path);
        self.assert_desecendants_exists(path)?;

        if self.entries.get(&realpath).is_some() {
            return if ignore_if_exists {
                Ok(())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("file {} already exists", realpath.display()),
                ))
            };
        }

        let _guard = self.lock_path_for_write(path);
        self.entries.insert(
            realpath.clone(),
            ImfsEntry::File {
                permissions: perms.unwrap_or_default(),
                times: ImfsEntryTimes::now(),
            },
        );

        let data = data.unwrap_or_default();
        self.data.insert(realpath, Arc::new(data));

        Ok(())
    }

    // #[tracing::instrument(skip(self))]
    pub fn open_file(
        self: Arc<Self>,
        path: &ImfsPath,
        options: &mut crate::OpenOptions,
    ) -> io::Result<crate::File> {
        let realpath = self.realpath(path);
        self.assert_desecendants_exists(&realpath)?;

        if options.create_new {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "In-memory file system does not support create_new",
            ));
        }

        // recreate file if it doesn't exists at the moment.
        if options.create {
            self.create_file(&realpath, None, None, options.truncate || options.read)?;
        }

        let Some(mut entry) = self.entries.get_mut(&realpath) else {
            return super::utils::not_found(path);
        };

        let ImfsEntry::File { permissions, times } = entry.value_mut() else {
            return super::utils::must_be_file(path);
        };

        let needs_writing = options.append
            || options.create
            || options.create_new
            || options.write
            || options.truncate;

        let needs_reading = options.read;

        // conflicting parameters, we should reject this one
        if needs_reading && needs_writing {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "tried to open file {} with reading and writing enabled",
                    path.display()
                ),
            ));
        }

        if needs_reading && !permissions.can_read() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "tried to read file {} without reading permission",
                    path.display()
                ),
            ));
        }

        if needs_writing && !permissions.can_write() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "tried to write file {} without writing permission",
                    path.display()
                ),
            ));
        }

        // checking for the parameters for the OpenOptions
        if needs_writing && options.append && options.truncate {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "tried to write file {} with both append and truncate options enabled",
                    path.display()
                ),
            ));
        }

        if needs_reading {
            let guard = self.lock_path_for_read(&realpath);
            let content = self
                .data
                .get(&realpath)
                .expect("missing data")
                .value()
                .clone();

            times.accessed = SystemTime::now();
            drop(entry);

            return Ok(crate::File::new(ReadableFile {
                content,
                _guard: guard,
                path: realpath,
                position: 0,
                vfs_handle: self.clone(),
            }));
        }

        times.accessed = SystemTime::now();
        drop(entry);

        let content = self
            .data
            .get(&realpath)
            .expect("missing data")
            .value()
            .clone();

        let content = if options.append {
            let mut content = Cursor::new(content.as_ref().clone());
            content.seek(io::SeekFrom::End(0))?;
            content
        } else {
            // heh, truncate and create option do have the same behavior anyway :)
            Cursor::new(Vec::new())
        };

        Ok(crate::File::new(WritableFile {
            content,
            vfs_handle: self.clone(),
            path: realpath,
        }))
    }
}

#[derive(Debug)]
pub struct ReadableFile {
    content: Arc<Vec<u8>>,
    _guard: ImfsPathReadLockGuard,
    path: ImfsPathBuf,
    position: u64,
    vfs_handle: Arc<InMemoryFsImpl>,
}

impl ReadableFile {
    fn len(&self) -> u64 {
        self.content.len() as u64 - self.position
    }
}

impl VfsFile for ReadableFile {
    fn io_read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let amt = std::cmp::min(buf.len(), self.len() as usize);
        if amt == 1 {
            buf[0] = self.content[self.position as usize];
        } else {
            buf[..amt].copy_from_slice(
                &self.content.as_slice()[self.position as usize..self.position as usize + amt],
            );
        }
        self.position += amt as u64;
        Ok(amt)
    }

    fn io_flush(&mut self) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("cannot write file of {} (read-only)", self.path.display()),
        ))
    }

    fn io_seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match pos {
            io::SeekFrom::Start(offset) => self.position = offset,
            io::SeekFrom::Current(offset) => self.position = (self.position as i64 + offset) as u64,
            io::SeekFrom::End(offset) => {
                self.position = (self.content.len() as i64 + offset) as u64
            }
        }
        Ok(self.position)
    }

    fn io_write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("cannot write file of {} (read-only)", self.path.display()),
        ))
    }

    fn metadata(&self) -> io::Result<crate::Metadata> {
        self.vfs_handle.metadata(&self.path)
    }

    fn set_len(&self, _size: u64) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("cannot set length for {}", self.path.display()),
        ))
    }

    fn set_times(&self, new_times: crate::SetFileTimes) -> io::Result<()> {
        let Some(mut entry) = self.vfs_handle.entries.get_mut(&self.path) else {
            return super::utils::not_found(&self.path);
        };

        let entry = entry.value_mut();
        match entry {
            ImfsEntry::File { times, .. } => {
                times.accessed = new_times.accessed.unwrap_or(times.accessed);
                times.created = new_times.accessed.unwrap_or(times.created);
                times.modified = new_times.accessed.unwrap_or(times.modified);
            }
            _ => return super::utils::must_be_file(&self.path),
        }

        Ok(())
    }

    fn sync_all(&self) -> io::Result<()> {
        Ok(())
    }

    fn sync_data(&self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct WritableFile {
    content: Cursor<Vec<u8>>,
    vfs_handle: Arc<InMemoryFsImpl>,
    path: ImfsPathBuf,
}

impl WritableFile {
    // #[tracing::instrument(skip(self))]
    fn commit(&mut self) -> io::Result<()> {
        // tracing::trace!(path = %self.path.display(), "commiting changes from a file");

        let _guard = self.vfs_handle.lock_path_for_write(&self.path);

        let mut content = vec![];
        std::mem::swap(&mut content, self.content.get_mut());

        let mut data = self.vfs_handle.data.get_mut(&self.path).unwrap();
        let data = data.value_mut();

        let mut entry = self.vfs_handle.entries.get_mut(&self.path).unwrap();
        let entry = entry.value_mut();
        let ImfsEntry::File { times, .. } = entry else {
            panic!("unexpected directory from entry")
        };
        times.modified = SystemTime::now();
        *data = Arc::new(content);

        Ok(())
    }
}

impl VfsFile for WritableFile {
    fn io_read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("cannot read file of {} (write-only)", self.path.display()),
        ))
    }

    fn io_flush(&mut self) -> io::Result<()> {
        self.content.flush()
    }

    fn io_seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.content.seek(pos)
    }

    fn io_write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.content.write(buf)
    }

    fn metadata(&self) -> io::Result<crate::Metadata> {
        self.vfs_handle.metadata(&self.path)
    }

    fn set_len(&self, _size: u64) -> io::Result<()> {
        Ok(())
    }

    fn set_times(&self, new_times: crate::SetFileTimes) -> io::Result<()> {
        let Some(mut entry) = self.vfs_handle.entries.get_mut(&self.path) else {
            return super::utils::not_found(&self.path);
        };

        let entry = entry.value_mut();
        match entry {
            ImfsEntry::File { times, .. } => {
                times.accessed = new_times.accessed.unwrap_or(times.accessed);
                times.created = new_times.accessed.unwrap_or(times.created);
                times.modified = new_times.accessed.unwrap_or(times.modified);
            }
            _ => return super::utils::must_be_file(&self.path),
        }

        Ok(())
    }

    fn sync_all(&self) -> io::Result<()> {
        Ok(())
    }

    fn sync_data(&self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for WritableFile {
    fn drop(&mut self) {
        let _ = self.commit();
    }
}
