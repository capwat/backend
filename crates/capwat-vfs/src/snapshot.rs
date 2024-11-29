use crate::Permissions;
use std::collections::BTreeMap;

/// Represents as a builder for [`VfsSnapshot`].
#[derive(Debug)]
#[must_use]
pub struct VfsSnapshotDirBuilder {
    children: BTreeMap<String, VfsSnapshot>,
    permissions: Permissions,
}

impl VfsSnapshotDirBuilder {
    pub fn empty_dir() -> VfsSnapshotDirBuilder {
        VfsSnapshotDirBuilder {
            children: BTreeMap::new(),
            permissions: Permissions::writable(),
        }
    }

    pub fn empty_file(mut self, name: impl Into<String>) -> Self {
        self.children.insert(name.into(), VfsSnapshot::empty_file());
        self
    }

    pub fn directory(mut self, name: impl Into<String>, contents: VfsSnapshotDirBuilder) -> Self {
        self.children.insert(name.into(), contents.build());
        self
    }

    pub fn file(mut self, name: impl Into<String>, contents: impl Into<Vec<u8>>) -> Self {
        let snapshot = VfsSnapshot::file(contents.into());
        self.children.insert(name.into(), snapshot);
        self
    }

    pub fn permissions(self, permissions: Permissions) -> Self {
        Self {
            children: self.children,
            permissions,
        }
    }

    pub fn build(self) -> VfsSnapshot {
        VfsSnapshot::Directory {
            children: self.children,
            permissions: self.permissions,
        }
    }
}

/// Represents as a builder for creating files with file system backends.
#[derive(Debug)]
#[must_use]
pub enum VfsSnapshot {
    File {
        data: Vec<u8>,
        permissions: Permissions,
    },
    Directory {
        children: BTreeMap<String, VfsSnapshot>,
        permissions: Permissions,
    },
}

impl VfsSnapshot {
    pub fn empty_dir() -> Self {
        Self::Directory {
            children: BTreeMap::new(),
            permissions: Permissions::writable(),
        }
    }

    #[inline(always)]
    pub fn empty_file() -> Self {
        Self::File {
            data: Vec::new(),
            permissions: Permissions::writable(),
        }
    }

    pub fn file<C: Into<Vec<u8>>>(contents: C) -> Self {
        Self::File {
            data: contents.into(),
            permissions: Permissions::writable(),
        }
    }

    pub fn permissions(self, permissions: Permissions) -> Self {
        match self {
            Self::File { data, .. } => Self::File { data, permissions },
            Self::Directory { children, .. } => Self::Directory {
                children,
                permissions,
            },
        }
    }

    #[inline(always)]
    pub fn build_dir() -> VfsSnapshotDirBuilder {
        VfsSnapshotDirBuilder::empty_dir()
    }
}
