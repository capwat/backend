use capwat_error::ext::{NoContextResultExt, ResultExt};
use capwat_error::Result;
use capwat_utils::cache::StaticValueCache;
use sea_query::{Asterisk, Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::PgConnection;
use std::time::Duration;

use crate::instance::{InstanceAggregates, InstanceAggregatesIdent};

impl InstanceAggregates {
    #[tracing::instrument(skip_all, name = "db.instance.settings.get_local")]
    pub async fn get_local(conn: &mut PgConnection) -> Result<InstanceAggregates> {
        static CACHE: StaticValueCache<InstanceAggregates> =
            StaticValueCache::new(Duration::from_secs(1));

        if let Some(cached) = CACHE.get().await {
            Ok(cached)
        } else {
            let (sql, values) = Query::select()
                .column(Asterisk)
                .from(InstanceAggregatesIdent::InstanceAggregates)
                .and_where(Expr::col(InstanceAggregatesIdent::Id).eq(0))
                .build_sqlx(PostgresQueryBuilder);

            let fresh = sqlx::query_as_with::<_, Self, _>(&sql, values)
                .fetch_one(conn)
                .await
                .erase_context()
                .attach_printable("could not get local instance aggregates")?;

            CACHE.set(fresh.clone());
            Ok(fresh)
        }
    }
}
