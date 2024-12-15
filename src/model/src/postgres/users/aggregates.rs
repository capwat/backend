use capwat_error::ext::{NoContextResultExt, ResultExt};
use capwat_error::Result;
use sea_query::{Asterisk, Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::PgConnection;

use crate::id::UserId;
use crate::user::{UserAggregates, UserAggregatesIdent};

impl UserAggregates {
    #[tracing::instrument(skip_all, name = "db.users.find")]
    pub async fn find(conn: &mut PgConnection, id: UserId) -> Result<Option<Self>> {
        // SELECT * FROM users WHERE id = <id>
        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(UserAggregatesIdent::UserAggregates)
            .and_where(Expr::col(UserAggregatesIdent::Id).eq(id.0))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, Self, _>(&sql, values)
            .fetch_optional(conn)
            .await
            .erase_context()
            .attach_printable("could not find user aggregates by id")
    }
}
