use error_stack::{Result, ResultExt};
use figment::{
  providers::{Env, Format, Toml},
  Figment,
};
use serde::Deserialize;
use std::{path::PathBuf, time::Duration};
use thiserror::Error;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct Server {
  pub database_url: String,
  pub code: u64,
  #[serde(skip, default)]
  pub(crate) path: Option<PathBuf>,
}

impl capwat_common::AppConfig for Server {
  fn init() -> Result<Self, capwat_common::LoadError>
  where
    Self: Sized,
  {
    Self::from_env().change_context(capwat_common::LoadError)
  }

  fn hooked_paths(&self) -> Vec<PathBuf> {
    let mut vec = Vec::new();
    if let Some(path) = self.path.clone() {
      vec.push(path);
    }
    vec
  }
}

#[derive(Debug, Error)]
#[error("Failed to load config")]
pub struct LoadFailed;

impl Server {
  pub fn from_env() -> Result<Self, LoadFailed> {
    dotenvy::dotenv().ok();
    if let Some(path) = Self::find_path()? {
      Self::from_file(&path)
    } else {
      let config = Self::figment().extract::<Self>().change_context(LoadFailed)?;
      config.validate().change_context(LoadFailed)?;
      Ok(config)
    }
  }

  pub fn from_file(path: &PathBuf) -> Result<Self, LoadFailed> {
    let contents = fs_err::read_to_string(path).change_context(LoadFailed)?;

    let mut config =
      Self::figment().join(Toml::string(&contents)).extract::<Self>().change_context(LoadFailed)?;

    config.validate().change_context(LoadFailed)?;
    config.path = Some(path.clone());

    Ok(config)
  }

  #[must_use]
  pub fn figment() -> Figment {
    Figment::new().merge(Env::prefixed("TEST_").map(|v| match v.as_str() {
      "DATABASE_URL" => "database_url".into(),
      _ => v.as_str().replace('_', ".").into(),
    }))
  }

  pub fn find_path() -> Result<Option<PathBuf>, LoadFailed> {
    if let Some(file) = match dotenvy::var("CONFIG_FILE") {
      Ok(p) => Some(PathBuf::from(p)),
      Err(dotenvy::Error::Io(e)) => return Err(e).change_context(LoadFailed),
      Err(..) => None,
    } {
      Ok(Some(file))
    } else {
      let cwd = std::env::current_dir().change_context(LoadFailed)?.into_boxed_path();
      let mut cwd = Some(&*cwd);

      while let Some(dir) = cwd {
        let entry = dir.join("test.toml");
        let file_exists = std::fs::metadata(&entry).map(|meta| meta.is_file()).unwrap_or_default();
        if file_exists {
          return Ok(Some(entry));
        }
        cwd = dir.parent();
      }
      Ok(None)
    }
  }
}

async fn checker(cfg: capwat_common::Config<Server>) {
  let mut interval = tokio::time::interval(Duration::from_secs(3));
  loop {
    interval.tick().await;
    println!("{:?}", &*cfg.get().await);
  }
}

#[allow(clippy::unwrap_used)]
#[tokio::main]
async fn main() {
  tracing_subscriber::fmt().pretty().with_max_level(tracing::Level::DEBUG).init();
  let cfg = capwat_common::Config::<Server>::init().unwrap();
  let checker_cfg = cfg.clone();

  let watcher = capwat_common::watch(cfg);
  let checker = checker(checker_cfg);

  let (a, ..) = tokio::join!(watcher, checker);
  a.unwrap();
}
