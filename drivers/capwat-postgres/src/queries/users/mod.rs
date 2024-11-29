use capwat_error::ext::ResultExt;
use capwat_error::Result;
use capwat_model::diesel::schema::users;
use capwat_model::user::InsertUser;
use capwat_model::User;

use diesel::dsl::exists;
use diesel::query_dsl::methods::FilterDsl;
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension};
use diesel_async::RunQueryDsl;
use thiserror::Error;

use super::util::{coalesce, lower};
use crate::pool::PgConnection;

pub trait UserPgImpl {
    async fn find(conn: &mut PgConnection<'_>, id: i64) -> Result<Option<User>>;
    async fn find_by_login(conn: &mut PgConnection<'_>, entry: &str) -> Result<Option<User>>;

    async fn check_email_taken(conn: &mut PgConnection<'_>, email: &str) -> Result<bool>;
    async fn check_username_taken(conn: &mut PgConnection<'_>, name: &str) -> Result<bool>;
}

#[derive(Debug, Error)]
#[error("Could not insert user")]
pub struct InsertUserError;

pub trait InsertUserPgImpl {
    async fn create(&self, conn: &mut PgConnection<'_>) -> Result<User, InsertUserError>;
}

impl UserPgImpl for User {
    #[tracing::instrument(skip_all, name = "db.query.users.find")]
    async fn find(conn: &mut PgConnection<'_>, id: i64) -> Result<Option<User>> {
        users::table
            .filter(users::id.eq(id))
            .get_result(&mut *conn)
            .await
            .optional()
            .erase_context()
    }

    #[tracing::instrument(skip_all, name = "db.query.users.find")]
    async fn find_by_login(conn: &mut PgConnection<'_>, entry: &str) -> Result<Option<User>> {
        // be specific when it comes to names :)
        users::table
            .filter(
                lower(users::name)
                    .eq(entry)
                    // `_@_@_` is an invalid email anyway.
                    .or(lower(coalesce(users::email, "_@_@_")).eq(entry.to_lowercase())),
            )
            .get_result::<User>(conn)
            .await
            .optional()
            .erase_context()
    }

    #[tracing::instrument(skip_all, name = "db.query.users.is_email_taken")]
    async fn check_email_taken(conn: &mut PgConnection<'_>, email: &str) -> Result<bool> {
        diesel::select(exists(users::table.filter(users::email.eq(email))))
            .get_result::<bool>(&mut *conn)
            .await
            .erase_context()
    }

    #[tracing::instrument(skip_all, name = "db.query.users.is_username_taken")]
    async fn check_username_taken(conn: &mut PgConnection<'_>, name: &str) -> Result<bool> {
        diesel::select(exists(
            users::table.filter(lower(users::name).eq(name.to_lowercase())),
        ))
        .get_result::<bool>(&mut *conn)
        .await
        .erase_context()
    }
}

impl InsertUserPgImpl for InsertUser<'_> {
    #[tracing::instrument(skip_all, name = "db.query.users.insert")]
    async fn create(&self, conn: &mut PgConnection<'_>) -> Result<User, InsertUserError> {
        // set to `None` if the display name specified is empty
        let display_name = if self.display_name.map(|v| !v.is_empty()).unwrap_or_default() {
            self.display_name
        } else {
            None
        };

        diesel::insert_into(users::table)
            .values((
                users::name.eq(self.name),
                users::display_name.eq(display_name),
                users::email.eq(self.email),
                users::password_hash.eq(self.password_hash),
            ))
            .get_result(&mut *conn)
            .await
            .change_context(InsertUserError)
    }
}
