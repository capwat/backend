pub mod schema;

mod util {
    use diesel::sql_types::{Nullable, Text};

    diesel::define_sql_function!(fn coalesce(x: Nullable<Text>, y: Text) -> Text);
    diesel::define_sql_function!(fn nullif(x: Text, y: Text) -> Nullable<Text>);
    diesel::define_sql_function!(fn lower(x: Text) -> Text);
}

/// Capwat's own [`diesel::prelude`] module but it prefers [`diesel_async::RunQueryDsl`] instead of
/// [`diesel::RunQueryDsl`] used in [`diesel`] which is not executed with `.await`.
///
/// [`Future`]: std::future::Future
#[allow(unused)]
mod prelude {
    #[doc(inline)]
    pub use super::util::{coalesce, lower, nullif};
    #[doc(inline)]
    pub use capwat_db::pool::PgConnection;
    #[doc(inline)]
    pub use capwat_error::{ext::*, Result};
    #[doc(inline)]
    pub use diesel::{
        associations::{Associations, GroupedBy, Identifiable},
        connection::Connection,
        deserialize::{Queryable, QueryableByName},
        expression::IntoSql as _,
        expression::SelectableHelper,
        expression::{
            AppearsOnTable, BoxableExpression, Expression, IntoSql, Selectable,
            SelectableExpression,
        },
        expression_methods::*,
        insertable::Insertable,
        query_builder::{AsChangeset, DecoratableTarget},
        query_dsl::{BelongingToDsl, CombineDsl, JoinOnDsl, QueryDsl},
        query_source::SizeRestrictedColumn as _,
        query_source::{Column, JoinTo, QuerySource, Table},
        result::OptionalExtension,
    };
    #[doc(inline)]
    pub use diesel_async::{RunQueryDsl, SaveChangesDsl};
}

mod follower;
mod instance;
mod post;
mod users;
