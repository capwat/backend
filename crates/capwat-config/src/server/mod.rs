use capwat_error::ext::{NoContextResultExt, ResultExt};
use capwat_error::Result;
use capwat_macros::ConfigParts;
use capwat_utils::{env, is_running_in_docker};
use capwat_vfs::Vfs;
use doku::Document;
use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use thiserror::Error;

use self::hcaptcha::PartialHCaptcha;
use crate::db_pools::PartialDatabasePools;
use crate::logging::{Logging, PartialLogging};
use crate::{vars, ConfigParts, DatabasePools};

mod hcaptcha;
pub use self::hcaptcha::HCaptcha;

#[derive(Debug, Document, ConfigParts)]
#[config(attr(derive(Debug, Deserialize)))]
pub struct Server {
    /// Configuration for server logging.
    ///
    /// Please refer to the documentation for logging to see its documentation.
    #[config(as_struct, as_type = "PartialLogging")]
    pub logging: Logging,

    /// Configuration for hCaptcha (Capwat's preferred CAPTCHA service).
    ///
    /// Please refer to the documentation for hCaptcha to see its documentation.
    #[config(as_struct, as_type = "Option<PartialHCaptcha>")]
    pub hcaptcha: Option<HCaptcha>,

    /// Configuration for database pools.
    ///
    /// You can refer `database` field as `db` to make it easier to type.
    /// ```toml
    /// # you can only choose one alias either: `database` or `db`.
    /// [database.primary]
    /// url = "<somewhere>"
    ///
    /// [db.primary]
    /// url = "<somewhere>"
    /// ```
    ///
    /// Please refer to the documentation for database pools to
    /// see its documentation.
    #[config(as_struct, as_type = "PartialDatabasePools")]
    #[config(attr(serde(alias = "db")))]
    pub database: DatabasePools,

    /// **Environment variable**: `CAPWAT_SERVER_IP`
    ///
    /// The default value is depending on what environment a process is running:
    /// - If it is in a Docker container, the default IP address will be `0.0.0.0`
    /// - Otherwise, it will be `127.0.0.1` or `localhost`
    #[doku(example = "127.0.0.1")]
    pub ip: IpAddr,

    /// **Environment variable**: `CAPWAT_SERVER_PORT`
    ///
    /// The default value is `8080` if not set.
    #[doku(example = "8080")]
    pub port: u16,

    /// **Environment variable**: `CAPWAT_SERVER_WORKERS`
    ///
    /// Total amount of workers will the server will run.
    ///
    /// The default value will be:
    /// - If the total amount of cores in the CPU of where a process is running
    ///   is greater than `4`, it will use `4` cores.
    /// - Otherwise, it will use half of amount of cores in the CPU available.
    #[doku(example = "4")]
    #[config(as_type = "Option<NonZeroUsize>")]
    pub workers: usize,

    /// Configuration file path.
    #[config(ignore)]
    pub file_location: Option<PathBuf>,
}

impl Server {
    /// Loads the server configuration from the program's current
    /// environment variables only.
    pub fn from_env(vfs: &Vfs) -> Result<Self, ServerLoadError> {
        let partial = PartialServer::from_env()?;
        Self::from_partial(vfs, partial, None)
    }

    /// Loads the server test configuration.
    #[must_use]
    pub fn for_tests() -> Self {
        let partial_db = PartialDatabasePools {
            primary: crate::db_pools::pool::PartialDatabasePool {
                min_connections: None,
                max_connections: None,
                readonly_mode: None,
                url: Some("".into()),
            },
            replica: None,
            enforce_tls: None,
            connection_timeout: None,
            idle_timeout: None,
            statement_timeout: None,
        };

        let partial = PartialServer {
            database: partial_db,
            logging: PartialLogging::from_env().unwrap(),
            hcaptcha: None,
            ip: None,
            port: None,
            workers: None,
        };

        let vfs = Vfs::new(capwat_vfs::backend::InMemoryFs::new());
        Self::from_partial(&vfs, partial, None).expect("unable to load test server configuration")
    }

    /// Loads the server configuration from both the program's current
    /// environment variables or the config file.
    ///
    /// Config files are overwritten first if found along side with
    /// environment variables then the default values.
    pub fn from_maybe_file(vfs: &Vfs) -> Result<Self, ServerLoadError> {
        let config_file_path = env::var_opt(vars::SERVER_CONFIG_FILE)
            .change_context(ServerLoadError)?
            .map(PathBuf::from)
            .or_else(|| Self::locate(vfs, None));

        let mut partial = PartialServer::from_env()?;

        // Merge them and resolve conflicts :)
        if let Some(config_file_path) = config_file_path.as_ref() {
            let from_file =
                PartialServer::from_toml(vfs, config_file_path).attach_printable_lazy(|| {
                    format!(
                        "could not read config file of {}",
                        config_file_path.display()
                    )
                })?;

            partial = partial.merge(from_file);
        }

        Self::from_partial(vfs, partial, config_file_path)
    }

    /// Attempts to locate a server config file from the current directory
    /// or the given directory up to its root ancestor.
    ///
    /// It will look for a file that has name: `capwat.toml` file
    /// on each directory's ancestors.
    ///
    /// If it hasn't found yet, it will look for either in paths (by order):
    ///
    /// **For Unix systems**:
    /// - `/etc/capwat/config.toml` (For Unix systems)
    ///
    /// **For Windows systems**:
    /// - `%APPDATA%/capwat/config.toml`
    /// - `%USERPROFILE%/.capwat/config.toml`
    #[must_use]
    pub fn locate(vfs: &Vfs, directory: Option<&Path>) -> Option<PathBuf> {
        // use the current directory if not available
        let directory = match directory {
            Some(n) => n.to_path_buf(),
            None => vfs.current_dir().ok()?,
        };

        // don't attempt to search for it if the given path
        // is not actually a directory
        if !vfs.is_dir(&directory) {
            return None;
        }

        for ancestor in directory.ancestors() {
            let path = vfs.join_path(ancestor, "capwat.toml");
            if vfs.is_file(&path) {
                // sometimes windows will go crazy with their canonicalization
                // system, let's strip it off from its disk perspective?
                return if cfg!(windows) && vfs.is_using_std_backend() {
                    Some(path.to_path_buf())
                } else {
                    vfs.canonicalize(&path).ok()
                };
            }
        }

        // looking for alternative paths
        #[cfg(not(any(windows, unix)))]
        static ALT_PATHS: &[&str] = &[];

        #[cfg(all(not(windows), unix))]
        static ALT_PATHS: &[&str] = &["/etc/capwat/config.toml"];

        // Forward slashes are okay in that case because we're going to use
        // the actual file system to look for it! :)
        #[cfg(windows)]
        static ALT_PATHS: &[&str] = &[
            "%APPDATA%\\capwat\\config.toml",
            "%USERPROFILE%\\.capwat\\config.toml",
        ];

        // do not try to look for alternative paths if the vfs
        // backend is not from the real system
        if !vfs.is_using_std_backend() {
            return None;
        }

        for path in ALT_PATHS {
            if vfs.is_file(path) {
                return Some(PathBuf::from(path));
            }
        }

        None
    }

    /// Generates TOML data with default values of the entire
    /// server configuration and documentation using [`doku`].
    #[must_use]
    pub fn generate_docs() -> String {
        use doku::toml::{EnumsStyle, Spacing};

        let fmt = doku::toml::Formatting {
            spacing: Spacing {
                lines_between_scalar_field_comments: 1,
                lines_between_scalar_fields: 0,
                ..Default::default()
            },
            enums_style: EnumsStyle::Commented,
            ..Default::default()
        };

        doku::to_toml_fmt::<Self>(&fmt)
    }
}

impl Server {
    fn from_partial(
        vfs: &Vfs,
        partial: PartialServer,
        file_location: Option<PathBuf>,
    ) -> Result<Self, ServerLoadError> {
        let logging = Logging::from_partial(partial.logging)
            .change_context(ServerLoadError)
            .attach_printable("could not load logging configuration")?;

        let hcaptcha = partial
            .hcaptcha
            .map(HCaptcha::from_partial)
            .transpose()
            .change_context(ServerLoadError)
            .attach_printable("could not load hcaptcha configuration")?;

        let database = DatabasePools::from_partial(partial.database)
            .change_context(ServerLoadError)
            .attach_printable("could not load database pools configuration")?;

        let workers = partial
            .workers
            .map(|v| v.get())
            .unwrap_or_else(Self::default_workers);

        let ip = partial.ip.unwrap_or_else(|| Self::default_ip(vfs));
        let port = partial.port.unwrap_or(8080);

        Ok(Self {
            logging,
            hcaptcha,
            database,
            ip,
            port,
            workers,
            file_location,
        })
    }

    fn default_workers() -> usize {
        let cores = num_cpus::get();
        if cores > 4 {
            4
        } else {
            (cores / 2).min(1)
        }
    }

    fn default_ip(vfs: &Vfs) -> IpAddr {
        IpAddr::V4(if is_running_in_docker(vfs) {
            Ipv4Addr::new(0, 0, 0, 0)
        } else {
            Ipv4Addr::new(127, 0, 0, 1)
        })
    }
}

#[derive(Debug, Error)]
#[error("Could not load configuration to start the server")]
pub struct ServerLoadError;

impl PartialServer {
    pub fn from_env() -> Result<Self, ServerLoadError> {
        let logging = PartialLogging::from_env().change_context(ServerLoadError)?;
        let hcaptcha = PartialHCaptcha::from_env().change_context(ServerLoadError)?;
        let database = PartialDatabasePools::from_env().change_context(ServerLoadError)?;
        let ip = env::var_opt_parsed::<IpAddr>(vars::SERVER_IP).change_context(ServerLoadError)?;
        let port = env::var_opt_parsed::<u16>(vars::SERVER_PORT).change_context(ServerLoadError)?;

        let workers = env::var_opt_parsed::<NonZeroUsize>(vars::SERVER_WORKERS)
            .change_context(ServerLoadError)?;

        Ok(Self {
            logging,
            hcaptcha,
            database,
            ip,
            port,
            workers,
        })
    }

    pub fn from_toml(vfs: &Vfs, path: &Path) -> Result<Self, ServerLoadError> {
        // we need that for errors soon :)
        let source = vfs.read_to_string(path).change_context(ServerLoadError)?;
        let document = crate::toml_internals::parse_document(&source)
            .map_err(|e| crate::toml_internals::emit_diagnostic(e, &source, path))
            .change_context(ServerLoadError)?;

        let data = crate::toml_internals::deserialize::<Self>(&document)
            .map_err(|e| crate::toml_internals::emit_diagnostic(e, &source, path))
            .change_context(ServerLoadError)?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests;
