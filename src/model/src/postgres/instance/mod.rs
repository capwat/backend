use capwat_error::ext::{NoContextResultExt, ResultExt};
use capwat_error::Result;
use capwat_utils::cache::StaticValueCache;
use sea_query::{Asterisk, Expr, ExprTrait, OnConflict, PostgresQueryBuilder, Query};
use sea_query_binder::{SqlxBinder, SqlxValues};
use sqlx::PgConnection;
use std::time::Duration;
use thiserror::Error;

use crate::id::InstanceId;
use crate::instance::{InstanceSettingsIdent, RegistrationMode, UpdateInstanceSettings};
use crate::InstanceSettings;

mod aggregates;

impl InstanceSettings {
    #[tracing::instrument(skip_all, name = "db.instance.settings.get_local")]
    pub async fn get_local(conn: &mut PgConnection) -> Result<InstanceSettings> {
        static CACHE: StaticValueCache<InstanceSettings> =
            StaticValueCache::new(Duration::from_secs(1));

        if let Some(cached) = CACHE.get().await {
            Ok(cached)
        } else {
            let (sql, values) = Query::select()
                .column(Asterisk)
                .from(InstanceSettingsIdent::InstanceSettings)
                .and_where(Expr::col(InstanceSettingsIdent::Id).eq(0))
                .build_sqlx(PostgresQueryBuilder);

            let fresh = sqlx::query_as_with::<_, Self, _>(&sql, values)
                .fetch_one(conn)
                .await
                .erase_context()
                .attach_printable("could not get local instance settings")?;

            CACHE.set(fresh.clone());
            Ok(fresh)
        }
    }

    #[tracing::instrument(skip_all, name = "db.instance.settings.get_local")]
    pub async fn setup_local(conn: &mut PgConnection) -> Result<Self> {
        let (sql, values) = Query::insert()
            .into_table(InstanceSettingsIdent::InstanceSettings)
            .columns([InstanceSettingsIdent::Id])
            .values_panic([0.into()])
            .on_conflict(
                OnConflict::new()
                    .expr(Expr::col(InstanceSettingsIdent::Id))
                    .value(InstanceSettingsIdent::Id, 0)
                    .to_owned(),
            )
            .returning_all()
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, Self, _>(&sql, values)
            .fetch_one(conn)
            .await
            .erase_context()
            .attach_printable("could not setup local instance settings")
    }
}

#[derive(Debug, Error)]
#[error("Could not update instance settings")]
pub struct UpdateInstanceSettingsError;

impl UpdateInstanceSettings {
    fn make_changeset(&self, compare: InstanceId) -> (String, SqlxValues) {
        let mut query = Query::update();
        self.make_changeset_sql(&mut query);

        if let Some(value) = self.registration_mode {
            query.value(
                InstanceSettingsIdent::RegistrationMode,
                Expr::value(match value {
                    RegistrationMode::Open => "open",
                    RegistrationMode::Closed => "closed",
                    RegistrationMode::RequireApproval => "require-approval",
                })
                .as_enum(super::RegistrationMode),
            );
        }

        query
            .and_where(Expr::col(InstanceSettingsIdent::Id).eq(compare.0))
            .returning_all()
            .build_sqlx(PostgresQueryBuilder)
    }

    #[tracing::instrument(skip_all, name = "db.instance.settings.update_local")]
    pub async fn perform_local(
        &self,
        conn: &mut PgConnection,
    ) -> Result<InstanceSettings, UpdateInstanceSettingsError> {
        let (sql, values) = self.make_changeset(InstanceId(0));
        sqlx::query_as_with::<_, InstanceSettings, _>(&sql, values)
            .fetch_one(conn)
            .await
            .change_context(UpdateInstanceSettingsError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use capwat_db::pool::PgPooledConnection;

    #[capwat_macros::postgres_query_test]
    async fn should_update(mut conn: PgPooledConnection) -> Result<()> {
        InstanceSettings::setup_local(&mut conn).await?;
        let new_settings = UpdateInstanceSettings::builder()
            .registration_mode(RegistrationMode::RequireApproval)
            .require_captcha(true)
            .require_email_registration(true)
            .require_email_verification(true)
            .post_max_characters(1000)
            .build()
            .perform_local(&mut conn)
            .await?;

        assert_eq!(
            new_settings.registration_mode,
            RegistrationMode::RequireApproval
        );
        assert!(new_settings.require_captcha);
        assert!(new_settings.require_email_registration);
        assert!(new_settings.require_email_verification);
        assert_eq!(new_settings.post_max_characters, 1000);

        Ok(())
    }

    #[capwat_macros::postgres_query_test]
    async fn should_get_local(mut conn: PgPooledConnection) -> Result<()> {
        InstanceSettings::setup_local(&mut conn).await?;
        InstanceSettings::get_local(&mut conn).await?;

        Ok(())
    }

    #[capwat_macros::postgres_query_test]
    async fn should_setup_local(mut conn: PgPooledConnection) -> Result<()> {
        InstanceSettings::setup_local(&mut conn).await?;

        Ok(())
    }
}
