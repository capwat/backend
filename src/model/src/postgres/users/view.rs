use capwat_error::ext::{NoContextResultExt, ResultExt};
use capwat_error::Result;
use sea_query::{
    Expr, ExprTrait, Func, Iden, IntoColumnRef, IntoIden, PostgresQueryBuilder, Query,
    SelectStatement, TableRef,
};
use sea_query_binder::SqlxBinder;
use sqlx::PgConnection;

use crate::id::UserId;
use crate::postgres::into_view_aliases;
use crate::user::{UserAggregates, UserAggregatesIdent, UserIdent, UserView};
use crate::User;

#[derive(Debug, Clone, Iden)]
struct U;

#[derive(Debug, Clone, Iden)]
struct A;

impl UserView {
    #[tracing::instrument(skip_all, name = "db.user_view.find")]
    pub async fn find(conn: &mut PgConnection, id: UserId) -> Result<Option<Self>> {
        let (sql, values) = Self::generate_select_stmt()
            .and_where(Expr::col((U, UserIdent::Id)).eq(id.0))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, Self, _>(&sql, values)
            .fetch_optional(conn)
            .await
            .erase_context()
            .attach_printable("could not find user view from user id")
    }

    #[tracing::instrument(skip_all, name = "db.user_view.find_by_login")]
    pub async fn find_by_login(conn: &mut PgConnection, entry: &str) -> Result<Option<Self>> {
        // they should have checked if it is actually an email
        assert_ne!(entry, "_@_@_@_");

        let (sql, values) = Self::generate_select_stmt()
            .and_where(
                Func::lower(Expr::col(UserIdent::Name))
                    .eq(entry.to_lowercase())
                    .or(Func::lower(Func::coalesce([
                        Expr::col(UserIdent::Email).into(),
                        Expr::val("_@_@_@_").into(),
                    ]))
                    .eq(entry.to_lowercase())),
            )
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, Self, _>(&sql, values)
            .fetch_optional(conn)
            .await
            .erase_context()
            .attach_printable("could not find user view from their login credentials")
    }

    fn generate_select_stmt() -> SelectStatement {
        Query::select()
            .exprs(into_view_aliases(User::make_view_columns(U).into_iter()))
            .exprs(into_view_aliases(
                UserAggregates::make_view_columns(A).into_iter(),
            ))
            .from_as(UserIdent::Users, U)
            .left_join(
                TableRef::Table(UserAggregatesIdent::UserAggregates.into_iden()).alias(A),
                Expr::col((A, UserAggregatesIdent::Id)).eq(Expr::col((U, UserIdent::Id))),
            )
            .group_by_columns([
                (U, UserIdent::Id).into_column_ref(),
                (A, UserAggregatesIdent::Id).into_column_ref(),
            ])
            .take()
    }
}

#[cfg(test)]
mod tests {
    use capwat_db::PgPooledConnection;
    use capwat_error::Result;

    use crate::postgres::users::tests::generate_alice;
    use crate::user::UserView;

    #[capwat_macros::postgres_query_test]
    async fn test_user_view(mut conn: PgPooledConnection) -> Result<()> {
        let (alice, _) = generate_alice(&mut conn).await?;

        let view = UserView::find(&mut conn, alice.id).await?.unwrap();
        assert_eq!(alice, view.user);

        Ok(())
    }
}
