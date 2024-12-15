mod protected_string;
mod sensitive;

pub mod cache;
pub mod env;
pub mod future;
pub mod serde_exts;

/// This value determines whether it was compiled in release mode
/// during building a binary or library.
pub const RELEASE: bool = cfg!(release);

pub use self::protected_string::{ProtectedString, ProtectedUrl};
pub use self::sensitive::Sensitive;

//////////////////////////////////////////////////////
use capwat_vfs::Vfs;

#[must_use]
pub fn is_running_in_docker(vfs: &Vfs) -> bool {
    // https://stackoverflow.com/a/23558932/23025722
    let proc_1_group = vfs
        .read_to_string("/proc/1/group")
        .map(|content| content.contains("/docker/"))
        .unwrap_or_default();

    let proc_mount = vfs
        .read_to_string("/proc/self/mountinfo")
        .map(|content| content.contains("/docker/"))
        .unwrap_or_default();

    vfs.exists("/.dockerenv") || vfs.exists("/run/.containerenv") || proc_1_group || proc_mount
}

/// This function yields the current thread until one of the exit
/// signals listed depending on the operating system that a host
/// machine is running on is triggered.
///
/// It allows programs to implement graceful shutdown to prevent
/// from any data loss or unexpected behavior to the server.
///
/// **Signals**:
/// - **For Windows / unsupported platforms**: It detects if `CTRL+C` is triggered
///
/// - **For Unix systems**: It detects whether `SIGINT` or `SIGTERM` is triggered
pub async fn shutdown_signal() {
    #[cfg(not(unix))]
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");

    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigint = signal(SignalKind::interrupt()).expect("failed to install SIGINT handler");
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");

        tokio::select! {
            _ = sigint.recv() => {},
            _ = sigterm.recv() => {},
        };
    }
}

pub mod time {
    use capwat_error::{ext::ResultExt, Result};
    use fundu::DurationParser;
    use std::time::Duration;

    pub fn parse_from_human_duration(s: &str) -> Result<Duration> {
        use fundu::TimeUnit;

        const PARSER: DurationParser<'static> = DurationParser::builder()
            .time_units(&[
                TimeUnit::MilliSecond,
                TimeUnit::Second,
                TimeUnit::Minute,
                TimeUnit::Hour,
                TimeUnit::Day,
            ])
            .allow_time_unit_delimiter()
            .disable_exponent()
            .build();

        let parsed = PARSER.parse(s)?;
        Duration::try_from(parsed).erase_context()
    }
}
