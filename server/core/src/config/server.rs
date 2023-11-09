use error_stack::{Report, Result, ResultExt};
use figment::providers::Format;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use validator::Validate;

use super::LoadError;
use crate::util::shims::{FigmentErrorAttachable, IntoValidatorReport};

#[derive(Debug, Deserialize, Validate)]
pub struct Server {
    #[serde(default)]
    #[validate(nested)]
    pub(crate) auth: super::Auth,
    #[validate(nested)]
    pub(crate) db: super::Database,
    #[serde(skip, default)]
    pub(crate) path: Option<PathBuf>,
}

impl Server {
    /// Loads server config from the shell environment where
    /// a binary depend on this library ran with its environment
    /// variables.
    ///
    /// If `WHIM_CONFIG_FILE` environment variable is set, it will try
    /// and load server config with a file from `WHIM_CONFIG_FILE`
    /// variable. If it is not set, it will try to find `whim.toml`
    /// file from the current directory and its descendants.
    pub fn from_env() -> Result<Self, LoadError> {
        dotenvy::dotenv().ok();

        if let Some(path) = Self::find_path().change_context(LoadError)? {
            Self::from_file(path)
        } else {
            let mut config = Self::figment()
                .extract::<Self>()
                .map_err(|e| Report::new(LoadError).attach_figment_error(e))?;

            config
                .validate()
                .into_validator_report()
                .change_context(LoadError)?;

            // `std::env::current_dir()`` is already checked in the if expression
            let default_file_path = std::env::current_dir()
                .expect("Self::get_path() should be ran from the first if chain")
                .join(Self::DEFAULT_FILE_NAME);

            // This is to prevent any raw data loss if for example we try
            // to load JWTs with its original keys lost.
            let mut doc = toml_edit::Document::new();
            if config.override_toml(&mut doc) {
                std::fs::write(&default_file_path, doc.to_string())
                    .change_context(LoadError)
                    .attach_printable("attempt to make the config file")?;
                config.path = Some(default_file_path.clone());
            }

            Ok(config)
        }
    }

    /// Loads server config from a config file only and
    /// attempts to merge it with environment variables.
    ///
    /// *If config file is used, it will override the missing
    /// values and replace with default values provided from
    /// each structs.*
    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<Self, LoadError> {
        use figment::providers::Toml;

        // Compiler friendly function
        fn from_file(path: &Path) -> Result<Server, LoadError> {
            dotenvy::dotenv().ok();

            let contents = std::fs::read_to_string(path)
                .change_context(LoadError)
                .attach_printable_lazy(|| format!("with config file: {}", path.display()))?;

            let mut config = Server::figment()
                .join(Toml::string(&contents))
                .extract::<Server>()
                .map_err(|e| Report::new(LoadError).attach_figment_error(e))
                .attach_printable_lazy(|| format!("with config file: {}", path.display()))?;

            config
                .validate()
                .into_validator_report()
                .change_context(LoadError)
                .attach_printable_lazy(|| format!("with config file: {}", path.display()))?;

            config.path = Some(path.to_path_buf());

            // Load the raw document of the config file and
            // to override a config file.
            let mut doc = contents
                .parse::<toml_edit::Document>()
                .change_context(LoadError)
                .expect("figment should check the contents of TOML config file");

            if config.override_toml(&mut doc) {
                std::fs::write(&path, doc.to_string())
                    .change_context(LoadError)
                    .attach_printable("trying to overwrite the config file")
                    .attach_printable_lazy(|| format!("with config file: {}", path.display()))?;
            }

            Ok(config)
        }

        from_file(path.as_ref())
    }

    pub const DEFAULT_FILE_NAME: &str = "whim.toml";
    pub const FILE_ENV: &str = "WHIM_CONFIG_FILE";

    /// Scans for any available `whim.toml` file from the user's current
    /// directory and its descendants or the `WHIM_CONFIG_FILE` environment
    /// variable from the binary's shell environment.
    pub fn find_path() -> std::io::Result<Option<PathBuf>> {
        if let Some(file) = match dotenvy::var(Self::FILE_ENV) {
            Ok(p) => Some(PathBuf::from(p)),
            Err(dotenvy::Error::Io(e)) => return Err(e),
            Err(..) => None,
        } {
            Ok(Some(file))
        } else {
            let cwd = std::env::current_dir()?.into_boxed_path();
            let mut cwd = Some(&*cwd);

            while let Some(dir) = cwd {
                let entry = dir.join(Self::DEFAULT_FILE_NAME);
                let file_exists = std::fs::metadata(&entry)
                    .map(|meta| meta.is_file())
                    .unwrap_or_default();

                if file_exists {
                    return Ok(Some(entry));
                }

                cwd = dir.parent();
            }

            Ok(None)
        }
    }
}

impl Server {
    pub const fn auth(&self) -> &super::Auth {
        &self.auth
    }

    pub const fn db(&self) -> &super::Database {
        &self.db
    }

    /// Gets the config file path of `whim.toml`.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

impl Server {
    /// Creates a base [`Figment`] object for [`Server`] to load bare
    /// server configuration. This function is there for implementing
    /// loader functions and testing.
    fn figment() -> figment::Figment {
        use figment::{providers::Env, Figment};

        Figment::new()
            // One big con about figment (env provider to be specific) especially
            // these fields with underscore in it.
            .merge(Env::prefixed("WHIM_").map(|v| match v.as_str() {
                "DB_PRIMARY_MIN_IDLE" => "db.primary.min_idle".into(),
                "DB_PRIMARY_POOL_SIZE" => "db.primary.pool_size".into(),

                "DB_REPLICA_MIN_IDLE" => "db.replica.min_idle".into(),
                "DB_REPLICA_POOL_SIZE" => "db.replica.pool_size".into(),

                "DB_ENFORCE_TLS" => "db.enforce_tls".into(),
                "DB_TIMEOUT_SECS" => "db.timeout_secs".into(),

                "AUTH_JWT_KEY" => "auth.jwt_key".into(),
                "AUTH_JWT_KEY_KEY" => "auth.jwt_key".into(),
                "JWT_KEY" => "auth.jwt_key".into(),
                "JWT_KEY_KEY" => "auth.jwt_key".into(),

                _ => v.as_str().replace("_", ".").into(),
            }))
            // Environment variable aliases
            .merge(Env::raw().map(|v| match v.as_str() {
                "DATABASE_URL" => "db.primary.url".into(),
                _ => v.into(),
            }))
    }

    fn override_toml(&self, tbl: &mut toml_edit::Table) -> bool {
        let mut overriden = false;
        if self.auth.jwt_key.is_generated() {
            let auth = tbl.entry("auth").or_insert(toml_edit::table());
            if let Some(auth) = auth.as_table_mut() {
                overriden = true;
                auth["jwt_key"] = toml_edit::value(&*self.auth.jwt_key);
                auth.key_decor_mut("jwt_key")
                    .unwrap()
                    .set_prefix("# Automatically generated by Whim. Any changes to the key\n# will result all users' crediential tokens will be invalid.\n");
            }
        }
        overriden
    }
}
