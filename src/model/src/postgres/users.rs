use capwat_db::pool::PgConnection;
use capwat_error::Result;
use diesel::dsl::exists;
use thiserror::Error;

use crate::id::UserId;
use crate::user::InsertUser;
use crate::User;

use super::prelude::*;
use super::schema::users;

impl User {
    #[tracing::instrument(skip_all, name = "db.users.find")]
    pub async fn find(conn: &mut PgConnection<'_>, id: UserId) -> Result<Option<User>> {
        users::table
            .filter(users::id.eq(id))
            .get_result::<User>(&mut *conn)
            .await
            .optional()
            .erase_context()
            .attach_printable("could not find user by id")
    }

    #[tracing::instrument(skip_all, name = "db.users.find_by_login")]
    pub async fn find_by_login(conn: &mut PgConnection<'_>, entry: &str) -> Result<Option<User>> {
        let filter = lower(users::name)
            .eq(entry.to_lowercase())
            .or(lower(coalesce(users::email, "_@_@_")).eq(entry.to_lowercase()));

        users::table
            .filter(filter)
            .get_result::<User>(&mut *conn)
            .await
            .optional()
            .erase_context()
            .attach_printable("could not find user by login credientials")
    }

    #[tracing::instrument(skip_all, name = "db.users.check_email_taken")]
    pub async fn check_email_taken(conn: &mut PgConnection<'_>, email: &str) -> Result<bool> {
        diesel::select(exists(users::table.filter(users::email.eq(email))))
            .get_result::<bool>(&mut *conn)
            .await
            .erase_context()
    }

    #[tracing::instrument(skip_all, name = "db.users.check_username_taken")]
    pub async fn check_username_taken(conn: &mut PgConnection<'_>, name: &str) -> Result<bool> {
        diesel::select(exists(
            users::table.filter(lower(users::name).eq(name.to_lowercase())),
        ))
        .get_result::<bool>(&mut *conn)
        .await
        .erase_context()
    }
}

#[derive(Debug, Error)]
#[error("Could not insert user")]
pub struct InsertUserError;

impl InsertUser<'_> {
    #[tracing::instrument(skip_all, name = "db.users.insert")]
    pub async fn insert(&self, conn: &mut PgConnection<'_>) -> Result<User, InsertUserError> {
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
                users::access_key_hash.eq(self.access_key_hash),
                users::encrypted_symmetric_key.eq(self.encrypted_symmetric_key),
                users::salt.eq(self.salt),
            ))
            .get_result::<User>(&mut *conn)
            .await
            .change_context(InsertUserError)
    }
}
