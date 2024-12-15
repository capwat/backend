use capwat_db::pool::PgConnection;
use capwat_error::ext::{NoContextResultExt, ResultExt};
use capwat_error::Result;
use sea_query::{Asterisk, Expr, ExprTrait, Func, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use thiserror::Error;

use crate::id::UserId;
use crate::user::{InsertUser, UpdateUser, UserIdent};
use crate::User;

impl User {
    #[tracing::instrument(skip_all, name = "db.users.find")]
    pub async fn find(conn: &mut PgConnection, id: UserId) -> Result<Option<User>> {
        // SELECT * FROM users WHERE id = <id>
        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(UserIdent::Users)
            .and_where(Expr::col(UserIdent::Id).eq(id.0))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, User, _>(&sql, values)
            .fetch_optional(conn)
            .await
            .erase_context()
            .attach_printable("could not find user by id")
    }

    #[tracing::instrument(skip_all, name = "db.users.find_by_login")]
    pub async fn find_by_login(conn: &mut PgConnection, entry: &str) -> Result<Option<User>> {
        // they should have checked if it is actually an email
        debug_assert_ne!(entry, "_@_@_@_");

        // SELECT * FROM users WHERE lower(name) = $1
        //     OR lower(coalesce(email, '_@_@_@_')) = $1
        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(UserIdent::Users)
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
            .attach_printable("could not find user by their login credientials")
    }

    #[tracing::instrument(skip_all, name = "db.users.check_email_taken")]
    pub async fn check_email_taken(conn: &mut PgConnection, email: &str) -> Result<bool> {
        // SELECT exists(SELECT * FROM users WHERE lower(email) = $1)
        let (sql, values) = Query::select()
            .expr(Expr::exists(
                Query::select()
                    .column(Asterisk)
                    .from(UserIdent::Users)
                    .and_where(Func::lower(Expr::col(UserIdent::Email)).eq(email.to_lowercase()))
                    .take(),
            ))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_scalar_with::<_, bool, _>(&sql, values)
            .fetch_one(conn)
            .await
            .erase_context()
    }

    #[tracing::instrument(skip_all, name = "db.users.check_username_taken")]
    pub async fn check_username_taken(conn: &mut PgConnection, name: &str) -> Result<bool> {
        // SELECT exists(SELECT * FROM users WHERE lower(name) = $1)
        let (sql, values) = Query::select()
            .expr(Expr::exists(
                Query::select()
                    .column(Asterisk)
                    .from(UserIdent::Users)
                    .and_where(Func::lower(Expr::col(UserIdent::Name)).eq(name.to_lowercase()))
                    .take(),
            ))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_scalar_with::<_, bool, _>(&sql, values)
            .fetch_one(conn)
            .await
            .erase_context()
    }
}

#[derive(Debug, Error)]
#[error("Could not update user")]
pub struct UpdateUserError;

impl UpdateUser<'_> {
    #[tracing::instrument(skip_all, name = "db.users.update")]
    pub async fn update(&self, conn: &mut PgConnection) -> Result<User, UpdateUserError> {
        let mut query = Query::update();
        self.make_changeset_sql(&mut query);

        let (sql, values) = query
            .and_where(Expr::col(UserIdent::Id).eq(self.id.0))
            .returning_all()
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, User, _>(&sql, values)
            .fetch_one(conn)
            .await
            .change_context(UpdateUserError)
    }
}

#[derive(Debug, Error)]
#[error("Could not insert user")]
pub struct InsertUserError;

impl InsertUser<'_> {
    #[tracing::instrument(skip_all, name = "db.users.insert")]
    pub async fn insert(&self, conn: &mut PgConnection) -> Result<User, InsertUserError> {
        // set to `None` if the display name specified is empty
        let display_name = if self.display_name.map(|v| !v.is_empty()).unwrap_or_default() {
            self.display_name
        } else {
            None
        };

        let (sql, values) = Query::insert()
            .into_table(UserIdent::Users)
            .columns([
                UserIdent::Name,
                UserIdent::DisplayName,
                UserIdent::Email,
                UserIdent::AccessKeyHash,
                UserIdent::EncryptedSymmetricKey,
                UserIdent::Salt,
            ])
            .values_panic([
                self.name.into(),
                display_name.into(),
                self.email.into(),
                self.access_key_hash.into(),
                self.encrypted_symmetric_key.into(),
                self.salt.into(),
            ])
            .returning_all()
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, User, _>(&sql, values)
            .fetch_one(conn)
            .await
            .change_context(InsertUserError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use capwat_crypto::client::RegisterUserParams;
    use capwat_db::pool::PgPooledConnection;

    async fn generate_alice(conn: &mut PgConnection) -> Result<(User, RegisterUserParams)> {
        let alice_params = capwat_crypto::client::generate_register_user_params(b"alice");
        let user = InsertUser::builder()
            .name("alice")
            .email("alice@example.com")
            .access_key_hash(&alice_params.access_key_hash.encode())
            .encrypted_symmetric_key(&alice_params.encrypted_symmetric_key.encode())
            .salt(&alice_params.salt.to_string())
            .build()
            .insert(conn)
            .await?;

        Ok((user, alice_params))
    }

    #[capwat_macros::postgres_query_test]
    async fn check_username_taken(mut conn: PgPooledConnection) -> Result<()> {
        assert!(!User::check_username_taken(&mut conn, "alice").await?);

        generate_alice(&mut conn).await?;
        assert!(User::check_username_taken(&mut conn, "alice").await?);

        Ok(())
    }

    #[capwat_macros::postgres_query_test]
    async fn check_email_taken(mut conn: PgPooledConnection) -> Result<()> {
        assert!(!User::check_email_taken(&mut conn, "alice@example.com").await?);

        generate_alice(&mut conn).await?;
        assert!(User::check_email_taken(&mut conn, "alice@example.com").await?);

        Ok(())
    }

    #[capwat_macros::postgres_query_test]
    async fn should_find_by_id(mut conn: PgPooledConnection) -> Result<()> {
        let (alice, _) = generate_alice(&mut conn).await?;
        let result = User::find(&mut conn, alice.id).await?;
        assert!(result.is_some());

        Ok(())
    }

    #[capwat_macros::postgres_query_test]
    async fn should_find_by_login(mut conn: PgPooledConnection) -> Result<()> {
        generate_alice(&mut conn).await?;

        // it should also support case-insensitive cases
        let result = User::find_by_login(&mut conn, "Alice").await?;
        assert!(result.is_some());

        // it should also support case-insensitive cases
        let result = User::find_by_login(&mut conn, "Alice@EXample.com").await?;
        assert!(result.is_some());

        let result = User::find_by_login(&mut conn, "").await?;
        assert!(result.is_none());

        Ok(())
    }

    #[capwat_macros::postgres_query_test]
    async fn should_update(mut conn: PgPooledConnection) -> Result<()> {
        let (old_alice, _) = generate_alice(&mut conn).await?;

        // bob must not get affected
        let bob = InsertUser::builder()
            .name("bob")
            .access_key_hash("a")
            .encrypted_symmetric_key("a")
            .salt("a")
            .build()
            .insert(&mut conn)
            .await?;

        let new_alice = UpdateUser::builder()
            .id(old_alice.id)
            .email_verified(true)
            .admin(true)
            .build()
            .update(&mut conn)
            .await?;

        assert_ne!(new_alice.admin, old_alice.admin);
        assert_ne!(new_alice.email_verified, old_alice.email_verified);
        assert_eq!(User::find(&mut conn, bob.id).await?.as_ref(), Some(&bob));

        Ok(())
    }

    #[capwat_macros::postgres_query_test]
    async fn should_insert(mut conn: PgPooledConnection) -> Result<()> {
        let (user, alice_params) = generate_alice(&mut conn).await?;

        assert_eq!(user.name, "alice");
        assert!(!user.admin);
        assert_eq!(user.display_name, None);
        assert_eq!(user.email, Some("alice@example.com".into()));
        assert!(!user.email_verified);
        assert_eq!(user.access_key_hash, alice_params.access_key_hash.encode());
        assert_eq!(
            user.encrypted_symmetric_key,
            alice_params.encrypted_symmetric_key.encode()
        );
        assert_eq!(user.salt, alice_params.salt.to_string());

        Ok(())
    }
}
