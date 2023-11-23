use std::{path::PathBuf, sync::Arc, time::Duration};

use error_stack::{Result, ResultExt};
// Less serious code of https://github.com/capwat/backend/blob/ba8caa40e1f68dd5e4bf4a9acc7e8b1b1822f5e3/src/config/server.rs
use figment::{
  providers::{Env, Format, Toml},
  Figment,
};
use notify::{RecommendedWatcher, Watcher};
use serde::Deserialize;
use thiserror::Error;
use tokio::{
  sync::{
    mpsc::{unbounded_channel, UnboundedReceiver},
    RwLock,
  },
  time::interval,
};
use validator::Validate;

#[derive(Debug, Error)]
#[error("Failed to load config")]
pub struct LoadFailed;

#[derive(Debug, Deserialize, Validate)]
pub struct Config {
  pub database_url: String,
  pub code: u64,
  #[serde(skip, default)]
  pub(crate) path: Option<PathBuf>,
}

impl Config {
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

#[allow(clippy::unwrap_used)]
fn establish_watcher(
) -> notify::Result<(RecommendedWatcher, UnboundedReceiver<notify::Result<notify::Event>>)> {
  let (tx, rx) = unbounded_channel();

  let watcher = RecommendedWatcher::new(
    move |res| {
      tx.send(res).unwrap();
    },
    notify::Config::default(),
  )?;

  Ok((watcher, rx))
}

async fn waiter(cfg: Arc<RwLock<Config>>) -> notify::Result<()> {
  let mut interval = interval(Duration::from_secs(3));
  loop {
    interval.tick().await;
    if let Ok(cfg) = cfg.try_read() {
      println!("{cfg:?}");
    } else {
      println!("LOCKED!");
    }
  }
}

async fn watch(cfg: Config) -> notify::Result<()> {
  if let Some(path) = cfg.path.clone() {
    let (mut watcher, mut rx) = establish_watcher()?;
    watcher.watch(&path, notify::RecursiveMode::NonRecursive)?;

    let cfg = Arc::new(RwLock::new(cfg));
    let waiter_cfg = cfg.clone();

    let (waiter_tx, mut waiter_rx) = unbounded_channel();
    let thread = std::thread::spawn(move || {
      if let Err(err) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime")
        .block_on(waiter(waiter_cfg))
      {
        eprintln!("waiter failed {err:?}");
        waiter_tx.send(()).expect("failed to send message");
      }
    });

    loop {
      tokio::select! {
        Some(res) = rx.recv() => {
          match res {
            Ok(..) => {
              println!("Modifying config!");
              match Config::from_env() {
                Ok(new_cfg) => {
                  *cfg.write().await = new_cfg;
                },
                Err(err) => eprintln!("failed to load config: {err:?}"),
              };
            },
            Err(err) => eprintln!("notify error: {err}"),
          };
        }
        _ = waiter_rx.recv() => break,
        else => break,
      }
    }

    if !thread.is_finished() {
      // it's okay, there are no remaining async threads after the loop
      thread.join().expect("thread failed");
    }
  }
  Ok(())
}

#[allow(clippy::unwrap_used)]
fn main() {
  let cfg = Config::from_env().unwrap();
  tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    if let Err(e) = watch(cfg).await {
      println!("error: {e:?}");
    }
  });
}
