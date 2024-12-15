use capwat_model::InstanceSettings;

use crate::{extract::LocalInstanceSettings, App};

/// Directly gets recently updated the data of [`InstanceSettings`] from
/// the local instance provided from the database.
///
/// [`InstanceSettings`]: capwat_model::InstanceSettings
#[tracing::instrument(skip_all, name = "test_utils.local_settings.from_local")]
pub async fn from_local(app: &App) -> LocalInstanceSettings {
    let inner = InstanceSettings::get_local(&mut app.db_read().await.unwrap())
        .await
        .unwrap();

    LocalInstanceSettings::new(inner)
}
