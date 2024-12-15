use capwat_error::{ext::ResultExt, Result};
use chrono::NaiveDateTime;

use crate::App;

pub mod headers {
    use axum::http::{HeaderName, HeaderValue};
    use axum_extra::headers::{Error, Header};

    static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

    pub struct XRequestId(String);

    impl XRequestId {
        pub fn as_str(&self) -> &str {
            self.0.as_str()
        }
    }

    impl Header for XRequestId {
        fn name() -> &'static HeaderName {
            &X_REQUEST_ID
        }

        fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
        where
            I: Iterator<Item = &'i HeaderValue>,
        {
            values
                .next()
                .and_then(|value| value.to_str().ok())
                .map(|value| Self(value.to_string()))
                .ok_or_else(Error::invalid)
        }

        fn encode<E>(&self, values: &mut E)
        where
            E: Extend<HeaderValue>,
        {
            let value = HeaderValue::from_str(&self.0).unwrap();
            values.extend(std::iter::once(value));
        }
    }
}

#[tracing::instrument(skip_all, name = "util.is_matched_with_db_clock")]
pub async fn check_clocks(app: &App) -> Result<(bool, NaiveDateTime, NaiveDateTime)> {
    use chrono::{Duration, NaiveDateTime, Utc};

    // Our minimum threshold to determine if we're in the similar timestamp
    // because database transactions can take around from milliseconds
    // to couple of seconds.
    const THRESHOLD: Duration = Duration::minutes(1);

    let our_timestamp = Utc::now().naive_utc();
    let db_timestamp = sqlx::query_scalar::<_, NaiveDateTime>(r"SELECT now()::timestamp")
        .fetch_one(&mut *app.db_read().await?)
        .await
        .attach_printable("could not get current time from the database")?;

    let matched = db_timestamp <= our_timestamp + THRESHOLD;
    Ok((matched, our_timestamp, db_timestamp))
}

// This test is very crucial as we need UTC timestamps to accurately when a user
// is actually registered from UTC timezone.
#[capwat_macros::api_test]
async fn should_match_timing_for_system_and_db(app: crate::App) {
    let (matched, our_timestamp, db_timestamp) = check_clocks(&app).await.unwrap();
    if !matched {
        panic!(
            "Please check the clock for the testing PostgreSQL server and the system time\n\
            PostgreSQL server clock: {db_timestamp}\n\
            System clock: {our_timestamp}"
        )
    }
}
