use crate::queries::users;
use crate::PgPool;

use capwat_error::Result;

mod find_user_by_id {
    use super::*;
    use capwat_model::user::InsertUser;

    #[capwat_macros::postgres_query_test]
    async fn should_find_user() -> Result<()> {
        let pool = PgPool::build_for_tests().await;

        let mut conn = pool.acquire().await?;
        let form = InsertUser::builder()
            .name("test")
            .password_hash("foo")
            .build();

        let new_user = users::insert_user(&mut conn, form).await?;
        let result = users::find_user_by_id(&mut conn, new_user.id).await?;
        assert!(result.is_some());

        let result = users::find_user_by_id(&mut conn, new_user.id + 1).await?;
        assert!(result.is_none());

        Ok(())
    }
}

mod insert_user {
    use super::*;
    use capwat_model::user::InsertUser;

    #[capwat_macros::postgres_query_test]
    async fn should_reject_if_name_exists() -> Result<()> {
        let pool = PgPool::build_for_tests().await;

        let mut conn = pool.acquire().await?;
        let form = InsertUser::builder()
            .name("test1")
            .password_hash("foo")
            .build();

        users::insert_user(&mut conn, form).await?;

        let form = InsertUser::builder()
            .name("test1")
            .password_hash("foo")
            .build();

        assert!(users::insert_user(&mut conn, form).await.is_err());
        Ok(())
    }

    #[capwat_macros::postgres_query_test]
    async fn should_reject_if_name_is_too_short() -> Result<()> {
        let pool = PgPool::build_for_tests().await;

        let mut conn = pool.acquire().await?;
        let form = InsertUser::builder()
            .name("sh")
            .password_hash("foo")
            .build();

        assert!(users::insert_user(&mut conn, form).await.is_err());

        let form = InsertUser::builder()
            .name("she")
            .password_hash("foo")
            .build();

        assert!(users::insert_user(&mut conn, form).await.is_ok());
        Ok(())
    }

    #[capwat_macros::postgres_query_test]
    async fn should_reject_if_password_hash_is_empty() -> Result<()> {
        let pool = PgPool::build_for_tests().await;

        let mut conn = pool.acquire().await?;
        let form = InsertUser::builder().name("sh").password_hash("").build();
        assert!(users::insert_user(&mut conn, form).await.is_err());

        Ok(())
    }

    #[capwat_macros::postgres_query_test]
    async fn should_return_no_display_name_if_value_is_empty() -> Result<()> {
        let pool = PgPool::build_for_tests().await;
        let mut conn = pool.acquire().await?;

        let form = InsertUser::builder()
            .name("test1")
            .password_hash("hello")
            .display_name("")
            .build();

        let result = users::insert_user(&mut conn, form).await?;
        assert_eq!(result.display_name, None);

        Ok(())
    }
}

mod update_user {
    use super::*;
    use capwat_model::user::{InsertUser, UpdateUser};

    #[capwat_macros::postgres_query_test]
    async fn should_update() -> Result<()> {
        let pool = PgPool::build_for_tests().await;

        let mut conn = pool.acquire().await?;
        let form = InsertUser::builder()
            .name("test")
            .password_hash("foo")
            .build();

        let user = users::insert_user(&mut conn, form).await?;
        let form = UpdateUser::builder()
            .name("test1")
            .display_name(Some("hello"))
            .email("test@example.com")
            .id(user.id)
            .build();

        users::update_user(&mut conn, form).await?;

        let result = users::find_user_by_id(&mut conn, user.id).await?.unwrap();
        assert_eq!(result.name, "test1");
        assert_eq!(result.display_name, Some("hello".to_string()));
        assert_eq!(result.email, Some("test@example.com".to_string()));

        Ok(())
    }

    #[capwat_macros::postgres_query_test]
    async fn should_reject_if_user_not_found() -> Result<()> {
        let pool = PgPool::build_for_tests().await;

        let mut conn = pool.acquire().await?;
        let form = UpdateUser::builder()
            .name("test1")
            .display_name(None)
            .email("test@example.com")
            .id(120000000000000000)
            .build();

        assert!(users::update_user(&mut conn, form).await.is_err());

        Ok(())
    }

    #[capwat_macros::postgres_query_test]
    async fn should_update_display_to_none_if_explicitly_set() -> Result<()> {
        let pool = PgPool::build_for_tests().await;

        let mut conn = pool.acquire().await?;
        let form = InsertUser::builder()
            .name("test")
            .password_hash("foo")
            .build();

        let user = users::insert_user(&mut conn, form).await?;
        let form = UpdateUser::builder()
            .name("test1")
            .display_name(None)
            .email("test@example.com")
            .id(user.id)
            .build();

        users::update_user(&mut conn, form).await?;

        let result = users::find_user_by_id(&mut conn, user.id).await?.unwrap();
        assert_eq!(result.name, "test1");
        assert_eq!(result.display_name, None);
        assert_eq!(result.email, Some("test@example.com".to_string()));

        Ok(())
    }
}
