#![expect(deprecated)]
use capwat_error::{
    ext::{NoContextResultExt, ResultExt},
    middleware::impls::Context,
    Error, Result,
};
use capwat_vfs::Vfs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::OnceLock;
use thiserror::Error;

static IS_DOTENV_LOADED: OnceLock<()> = OnceLock::new();

#[derive(Debug, Error)]
#[error("Failed to load .env file")]
pub struct LoadDotenvError;

pub fn load_dotenv(vfs: &Vfs) -> Result<PathBuf, LoadDotenvError> {
    let _ = IS_DOTENV_LOADED.set(());

    let path = find_dotenv(vfs)
        .change_context(LoadDotenvError)
        .attach_printable("could not find `.env` file to load")?;

    let file = capwat_vfs::OpenOptions::new()
        .read(true)
        .open(vfs, &path)
        .change_context(LoadDotenvError)
        .attach_printable_lazy(|| format!("could not open file of {}", path.display()))?;

    dotenvy::Iter::new(file)
        .load()
        .change_context(LoadDotenvError)
        .attach_printable_lazy(|| {
            format!(
                "could not load environment variables with {}",
                path.display()
            )
        })?;

    Ok(path)
}

pub fn find_dotenv(vfs: &Vfs) -> Result<PathBuf> {
    // non-recursive way of finding .env files
    let current_dir = vfs.current_dir()?;
    for ancestor in current_dir.ancestors() {
        let candidate = ancestor.join(".env");
        match vfs.metadata(&candidate) {
            Ok(metadata) if metadata.is_file() => return Ok(candidate),
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                return Err(error)
                    .attach_printable(format!(
                        "failed to load metadata for {}",
                        candidate.display()
                    ))
                    .erase_context();
            }
            _ => {}
        };
    }

    let error = std::io::Error::new(std::io::ErrorKind::NotFound, "cannot find `.env` file");
    Err(error).erase_context()
}

#[derive(Debug, Error)]
#[error("Could not get value of an environment variable")]
pub struct VarError;

#[track_caller]
pub fn var(key: &str) -> Result<String> {
    preload();
    match std::env::var(key) {
        Ok(n) => Ok(n),
        Err(error) => Err(make_var_error(key, error)),
    }
}

#[track_caller]
pub fn var_parsed<T: FromStr>(key: &str) -> Result<T>
where
    T::Err: Context,
{
    let value = var(key)?;
    match value.parse() {
        Ok(n) => Ok(n),
        Err(error) => Err(Error::unknown_generic(error))
            .change_context(VarError)
            .attach_printable_lazy(|| format!("could not parse value of {key:?}"))
            .erase_context(),
    }
}

#[track_caller]
pub fn var_opt(key: &str) -> Result<Option<String>> {
    preload();
    match std::env::var(key) {
        Ok(n) => Ok(Some(n)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(make_var_error(key, error)),
    }
}

#[track_caller]
pub fn var_opt_parsed<T: FromStr>(key: &str) -> Result<Option<T>>
where
    T::Err: Context,
{
    let Some(value) = var_opt(key)? else {
        return Ok(None);
    };
    match value.parse() {
        Ok(n) => Ok(Some(n)),
        Err(error) => Err(Error::unknown_generic(error))
            .change_context(VarError)
            .attach_printable_lazy(|| format!("could not parse value of {key:?}"))
            .erase_context(),
    }
}

#[track_caller]
pub fn var_opt_parsed_fn<T, F: Fn(&str) -> capwat_error::Result<T>>(
    key: &str,
    parser: F,
) -> Result<Option<T>> {
    let Some(value) = var_opt(key)? else {
        return Ok(None);
    };
    match parser(&value) {
        Ok(n) => Ok(Some(n)),
        Err(error) => Err(error)
            .change_context(VarError)
            .attach_printable_lazy(|| format!("could not parse value of {key:?}"))
            .erase_context(),
    }
}

fn make_var_error(key: &str, error: std::env::VarError) -> Error {
    match error {
        std::env::VarError::NotPresent => {
            Error::unknown_generic(VarError).attach_printable(format!("{key:?} is missing"))
        }
        err @ std::env::VarError::NotUnicode(..) => Error::unknown_generic(err)
            .change_context_slient(VarError)
            .attach_printable(format!("{key:?} has an invalid UTF-8 value")),
    }
}

fn preload() {
    if IS_DOTENV_LOADED.get().is_none() {
        panic!(
            "Please call `crate::util::env::load_dotenv(..)` before utilizing env vars functions"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use capwat_vfs::{backend::InMemoryFs, VfsSnapshot};

    static DOTENV_FILE_1: &str = "__NGL_TEST__=hi!";
    static DOTENV_FILE_2: &str = "__NGL_TEST__=goodbye!";

    static EXPECTED_VALUE: &str = "hi!";
    static EXPECTED_VALUE_2: &str = "goodbye!";

    static ENV_VAR: &str = "__NGL_TEST__";

    fn init_vfs() -> Vfs {
        let snapshot = VfsSnapshot::build_dir()
            .directory(
                "a",
                VfsSnapshot::build_dir()
                    .directory(
                        "b",
                        VfsSnapshot::build_dir()
                            .directory("c", VfsSnapshot::build_dir().file(".env", DOTENV_FILE_1)),
                    )
                    .file(".env", DOTENV_FILE_2),
            )
            .build();

        Vfs::new(InMemoryFs::new().apply_snapshot("/", snapshot).unwrap())
    }

    #[test]
    fn test_load_dotenv() {
        let vfs = init_vfs();
        std::env::remove_var(ENV_VAR);

        assert!(load_dotenv(&vfs).is_err());

        vfs.set_current_dir("/a/b/c").unwrap();
        assert_eq!(load_dotenv(&vfs).unwrap(), PathBuf::from("/a/b/c/.env"));
        assert_eq!(std::env::var(ENV_VAR).unwrap(), EXPECTED_VALUE);
        std::env::remove_var(ENV_VAR);

        vfs.set_current_dir("/a/b").unwrap();
        assert_eq!(load_dotenv(&vfs).unwrap(), PathBuf::from("/a/.env"));
        assert_eq!(std::env::var(ENV_VAR).unwrap(), EXPECTED_VALUE_2);

        std::env::remove_var(ENV_VAR);
    }

    #[test]
    fn test_find_dotenv() {
        let vfs = init_vfs();
        assert!(find_dotenv(&vfs).is_err());

        vfs.set_current_dir("/a/b/c").unwrap();
        assert_eq!(find_dotenv(&vfs).unwrap(), PathBuf::from("/a/b/c/.env"));

        vfs.set_current_dir("/a/b").unwrap();
        assert_eq!(find_dotenv(&vfs).unwrap(), PathBuf::from("/a/.env"));
    }
}
