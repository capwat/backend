use capwat_error::{
    ext::{NoContextResultExt, ResultExt},
    Result,
};
use capwat_model::{
    diesel::schema::instance_settings,
    id::InstanceId,
    instance::{InstanceSettings, UpdateInstanceSettings},
};
use capwat_utils::cache::StaticValueCache;
use diesel::{query_dsl::methods::FilterDsl, ExpressionMethods};
use diesel_async::RunQueryDsl;
use std::time::Duration;
use thiserror::Error;

use crate::pool::PgConnection;

#[derive(Debug, Error)]
#[error("Could not setup local instance settings")]
pub struct SetupInstanceSettingsError;

#[derive(Debug, Error)]
#[error("Could not update instance settings")]
pub struct UpdateInstanceSettingsError;

pub trait InstanceSettingsPgImpl {
    async fn get_local(conn: &mut PgConnection<'_>) -> Result<InstanceSettings>;
    async fn setup_local(conn: &mut PgConnection<'_>) -> Result<(), SetupInstanceSettingsError>;

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
    #[tracing::instrument(skip_all, name = "query.instance_settings.get_local")]
    async fn get_local(conn: &mut PgConnection<'_>) -> Result<InstanceSettings> {
        static LOCAL_CACHE: StaticValueCache<InstanceSettings> =
            StaticValueCache::new(Duration::from_secs(1));

        if let Some(cached) = LOCAL_CACHE.get().await {
            Ok(cached)
        } else {
            let fresh = instance_settings::table
                .filter(instance_settings::id.eq(0))
                .get_result::<InstanceSettings>(&mut *conn)
                .await
                .erase_context()?;

            LOCAL_CACHE.set(fresh.clone());
            Ok(fresh)
        }
    }

    #[tracing::instrument(skip_all, name = "query.instance_settings.setup_local")]
    async fn setup_local(conn: &mut PgConnection<'_>) -> Result<(), SetupInstanceSettingsError> {
        diesel::insert_into(instance_settings::table)
            .values(instance_settings::id.eq(0))
            .on_conflict(instance_settings::id)
            .do_nothing()
            .returning(instance_settings::all_columns)
            .execute(&mut *conn)
            .await
            .erase_context()
            .change_context(SetupInstanceSettingsError)?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "query.instance_settings.update")]
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

    #[tracing::instrument(skip_all, name = "query.instance_settings.update_local")]
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
