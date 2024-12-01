mod protected_string;

pub mod env;
pub mod serde_exts;

// TODO: Remove this thing here i guess and probably implement
//       our own CacheLock with moka::Cache inside.
pub use moka;

/// This value determines whether it was compiled in release mode
/// during building a binary or library.
pub const RELEASE: bool = cfg!(release);

pub use self::protected_string::ProtectedString;
pub use capwat_api_types::util::Sensitive;

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
