use super::prelude::*;
use super::schema::instance_settings;

use capwat_utils::cache::StaticValueCache;
use std::time::Duration;
use thiserror::Error;

use crate::id::InstanceId;
use crate::instance::UpdateInstanceSettings;
use crate::InstanceSettings;

impl InstanceSettings {
    #[tracing::instrument(skip_all, name = "db.instance.settings.get_local")]
    pub async fn get_local(conn: &mut PgConnection<'_>) -> Result<InstanceSettings> {
        static CACHE: StaticValueCache<InstanceSettings> =
            StaticValueCache::new(Duration::from_secs(1));

        if let Some(cached) = CACHE.get().await {
            Ok(cached)
        } else {
            let fresh = instance_settings::table
                .filter(instance_settings::id.eq(0))
                .get_result::<InstanceSettings>(&mut *conn)
                .await
                .erase_context()
                .attach_printable("could not get local instance settings")?;

            CACHE.set(fresh.clone());
            Ok(fresh)
        }
    }

    #[tracing::instrument(skip_all, name = "db.instance.settings.get_local")]
    pub async fn setup_local(conn: &mut PgConnection<'_>) -> Result<Self> {
        diesel::insert_into(instance_settings::table)
            .values(instance_settings::id.eq(0))
            .on_conflict(instance_settings::id)
            .do_update()
            .set(instance_settings::id.eq(0))
            .returning(instance_settings::all_columns)
            .get_result(&mut *conn)
            .await
            .erase_context()
            .attach_printable("could not setup local instance settings")
    }
}

#[derive(Debug, Error)]
#[error("Could not update instance settings")]
pub struct UpdateInstanceSettingsError;

impl UpdateInstanceSettings {
    #[tracing::instrument(skip_all, name = "db.instance.settings.update")]
    pub async fn perform(
        &self,
        conn: &mut PgConnection<'_>,
        id: InstanceId,
    ) -> Result<InstanceSettings, UpdateInstanceSettingsError> {
        diesel::update(instance_settings::table)
            .set(self)
            .filter(instance_settings::id.eq(id))
            .get_result::<InstanceSettings>(&mut *conn)
            .await
            .change_context(UpdateInstanceSettingsError)
    }

    #[tracing::instrument(skip_all, name = "db.instance.settings.update_local")]
    pub async fn perform_local(
        &self,
        conn: &mut PgConnection<'_>,
    ) -> Result<InstanceSettings, UpdateInstanceSettingsError> {
        diesel::update(instance_settings::table)
            .set(self)
            .filter(instance_settings::id.eq(0))
            .get_result::<InstanceSettings>(&mut *conn)
            .await
            .change_context(UpdateInstanceSettingsError)
    }
}
