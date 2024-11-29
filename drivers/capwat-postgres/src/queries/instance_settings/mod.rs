use capwat_error::ext::ResultExt;
use capwat_error::Result;
use capwat_model::diesel::schema::instance_settings;
use capwat_model::id::InstanceId;
use capwat_model::instance_settings::{InstanceSettings, UpdateInstanceSettings};
use capwat_utils::moka;
use diesel::query_dsl::methods::FilterDsl;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use std::sync::LazyLock;
use thiserror::Error;
use tracing::trace;

use crate::pool::PgConnection;

#[derive(Debug, Error)]
#[error("Could not update instance settings")]
pub struct UpdateInstanceSettingsError;

pub trait InstanceSettingsPgImpl {
    async fn get_local(conn: &mut PgConnection<'_>) -> Result<InstanceSettings>;
    async fn setup_local(conn: &mut PgConnection<'_>) -> Result<()>;

    async fn update(
        conn: &mut PgConnection<'_>,
        id: InstanceId,
        form: &UpdateInstanceSettings,
    ) -> Result<InstanceSettings, UpdateInstanceSettingsError>;

    async fn update_local(
        conn: &mut PgConnection<'_>,
        form: &UpdateInstanceSettings,
    ) -> Result<InstanceSettings, UpdateInstanceSettingsError>;
}

impl InstanceSettingsPgImpl for InstanceSettings {
    #[tracing::instrument(skip_all, name = "db.query.instance_settings.get_local")]
    async fn get_local(conn: &mut PgConnection<'_>) -> Result<InstanceSettings> {
        trace!("getting local instance settings...");

        static LOCAL_CACHE: LazyLock<moka::future::Cache<(), InstanceSettings>> =
            LazyLock::new(|| {
                moka::future::Cache::builder()
                    .max_capacity(1)
                    .time_to_live(std::time::Duration::from_secs(1))
                    .build()
            });

        if let Some(cached) = LOCAL_CACHE.get(&()).await {
            trace!("cache hit! getting cached version of local instance settings");
            Ok(cached)
        } else {
            trace!("cache miss! getting local instance settings from DB");

            let fresh = instance_settings::table
                .filter(instance_settings::id.eq(0))
                .get_result::<InstanceSettings>(&mut *conn)
                .await
                .erase_context()?;

            LOCAL_CACHE.insert((), fresh.clone()).await;
            Ok(fresh)
        }
    }

    #[tracing::instrument(skip_all, name = "db.query.instance_settings.setup_local")]
    async fn setup_local(conn: &mut PgConnection<'_>) -> Result<()> {
        diesel::insert_into(instance_settings::table)
            .values(instance_settings::id.eq(0))
            .on_conflict(instance_settings::id)
            .do_nothing()
            .returning(instance_settings::all_columns)
            .execute(&mut *conn)
            .await
            .erase_context()?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "db.query.instance_settings.update")]
    async fn update(
        conn: &mut PgConnection<'_>,
        id: InstanceId,
        form: &UpdateInstanceSettings,
    ) -> Result<InstanceSettings, UpdateInstanceSettingsError> {
        diesel::update(instance_settings::table)
            .set(form)
            .filter(instance_settings::id.eq(id))
            .returning(instance_settings::all_columns)
            .get_result::<InstanceSettings>(&mut *conn)
            .await
            .change_context(UpdateInstanceSettingsError)
    }

    #[tracing::instrument(skip_all, name = "db.query.instance_settings.update_local")]
    async fn update_local(
        conn: &mut PgConnection<'_>,
        form: &UpdateInstanceSettings,
    ) -> Result<InstanceSettings, UpdateInstanceSettingsError> {
        diesel::update(instance_settings::table)
            .set(form)
            .filter(instance_settings::id.eq(0))
            .returning(instance_settings::all_columns)
            .get_result::<InstanceSettings>(&mut *conn)
            .await
            .change_context(UpdateInstanceSettingsError)
    }
}
