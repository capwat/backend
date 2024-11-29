use super::ImfsPath;
use super::{InMemoryFs, InMemoryFsImpl};

use crate::backend::imfs::ImfsEntry;
use crate::VfsSnapshot;

use std::io;

impl InMemoryFs {
    pub fn apply_snapshot<P: AsRef<ImfsPath>>(
        self,
        path: P,
        snapshot: VfsSnapshot,
    ) -> io::Result<Self> {
        self.handle.apply_snapshot(path.as_ref(), snapshot)?;
        Ok(self)
    }
}

impl InMemoryFsImpl {
    fn apply_snapshot(&self, path: &ImfsPath, snapshot: VfsSnapshot) -> io::Result<()> {
        // We could use its parent to set as a current directory
        if let Some(parent_path) = path.parent() {
            let mut current_dir = self.current_dir.write();
            if current_dir.is_none() {
                *current_dir = Some(parent_path.to_path_buf());
            }
            drop(current_dir);

            if let Some(mut entry) = self.entries.get_mut(parent_path) {
                match entry.value_mut() {
                    ImfsEntry::Directory { children, .. } => children.push(path.to_path_buf()),
                    ImfsEntry::File { .. } => return super::utils::must_be_dir(parent_path),
                };
            }
        }

        match snapshot {
            VfsSnapshot::File { data, permissions } => {
                self.create_file(path, Some(data), Some(permissions), false)?;
            }
            VfsSnapshot::Directory {
                children,
                permissions,
            } => {
                // We could use the directory as our new current directory
                let mut current_dir = self.current_dir.write();
                if current_dir.is_none() {
                    *current_dir = Some(path.to_path_buf());
                }
                drop(current_dir);

                self.create_dir(path, Some(permissions), false)?;
                for (child_name, snapshot) in children {
                    let full_path = path.join(child_name);
                    self.apply_snapshot(&full_path, snapshot)?;
                }
            }
        };

        Ok(())
    }
}
