use std::io;

// Because the way Windows represents their path system for backwards compatibility,
// we need to use this crate in order to have Unix-related path in InMemoryFs.
#[cfg(not(windows))]
pub use std::path::{Path as ImfsPath, PathBuf as ImfsPathBuf};

#[cfg(windows)]
pub use unix_path::{Path as ImfsPath, PathBuf as ImfsPathBuf};

#[inline(always)]
pub fn to_std_path(path: &ImfsPath) -> std::path::PathBuf {
    #[cfg(windows)]
    {
        let owned = path.to_string_lossy().to_string();
        std::path::PathBuf::from(owned)
    }
    #[cfg(not(windows))]
    {
        path.to_path_buf()
    }
}

#[inline(always)]
pub fn to_unix_path(path: &std::path::Path) -> ImfsPathBuf {
    #[cfg(windows)]
    {
        use path_slash::PathExt;
        let path = path.to_slash_lossy().to_string();
        ImfsPathBuf::from(path)
    }
    #[cfg(not(windows))]
    {
        path.to_path_buf()
    }
}

pub fn must_be_file<T>(path: &ImfsPath) -> io::Result<T> {
    Err(io::Error::new(
        io::ErrorKind::Other,
        format!(
            "path {} was a directory, but must be a file",
            path.display()
        ),
    ))
}

pub fn must_be_dir<T>(path: &ImfsPath) -> io::Result<T> {
    Err(io::Error::new(
        io::ErrorKind::Other,
        format!(
            "path {} was a file, but must be a directory",
            path.display()
        ),
    ))
}

pub fn not_found<T>(path: &ImfsPath) -> io::Result<T> {
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("path {} not found", path.display()),
    ))
}
