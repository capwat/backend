use error_stack::{Result, ResultExt};
use notify::{Event as WatchEvent, RecommendedWatcher, Result as WatchResult, Watcher};
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio::sync::{RwLock, RwLockReadGuard};

#[derive(Debug, Error)]
#[error("Failed to load configuration")]
pub struct LoadError;

pub struct Config<T: AppConfig>(Arc<RwLock<T>>);

impl<T: AppConfig> Config<T> {
  pub fn init() -> Result<Self, LoadError> {
    T::init().map(|v| Self(Arc::new(RwLock::new(v))))
  }

  pub async fn get(&self) -> RwLockReadGuard<'_, T> {
    self.0.read().await
  }
}

impl<T: AppConfig> Clone for Config<T> {
  fn clone(&self) -> Self {
    Self(Arc::clone(&self.0))
  }
}

pub trait AppConfig: Debug {
  fn init() -> Result<Self, LoadError>
  where
    Self: Sized;

  /// Gets a list of path/s that the hot reloading watcher
  /// must keep on eye when listening for new changes to
  /// the server configuration.
  ///
  /// Also, it must not empty!
  fn hooked_paths(&self) -> Vec<PathBuf>;
}

fn establish_fs_watcher(
) -> WatchResult<(RecommendedWatcher, UnboundedReceiver<WatchResult<WatchEvent>>)> {
  let (tx, rx) = unbounded_channel();

  let config = notify::Config::default().with_compare_contents(true);
  let watcher = RecommendedWatcher::new(
    move |res| {
      if let Err(err) = tx.send(res) {
        tracing::error!(error = ?err, "Failed to send event to the config watcher thread");
      }
    },
    config,
  )?;

  Ok((watcher, rx))
}

#[derive(Debug, Error)]
pub enum WatchError {
  #[error("Failed to initialize file watcher")]
  InitFailed,
}

#[tracing::instrument(skip(cfg))]
pub async fn watch<T: AppConfig>(cfg: Config<T>) -> Result<(), WatchError> {
  let paths = cfg.get().await.hooked_paths();
  if paths.is_empty() {
    return Ok(());
  }

  let (mut watcher, mut event_rx) =
    establish_fs_watcher().change_context(WatchError::InitFailed)?;

  for path in paths {
    watcher
      .watch(&path, notify::RecursiveMode::NonRecursive)
      .change_context(WatchError::InitFailed)
      .attach_printable_lazy(|| format!("path: {}", path.to_string_lossy()))?;
  }

  while let Some(event) = event_rx.recv().await {
    // We try to attempt to reload the config...
    match event {
      Ok(..) => {
        tracing::warn!("detected changes to the config; attempting to override changes");
        match T::init() {
          Ok(contents) => {
            let mut cfg = cfg.0.write().await;
            tracing::info!(new = ?contents, old = ?cfg, "new config loaded successfully; overriding config");
            *cfg = contents;
          },
          Err(err) => {
            tracing::error!(err = ?err, "failed to load new config; reverting back to the old config");
          },
        }
      },
      Err(err) => match err.kind {
        notify::ErrorKind::PathNotFound => {},
        _ => {
          tracing::error!(err = ?err, "Unexpected file watcher error occurred");
        },
      },
    }
  }

  Ok(())
}
