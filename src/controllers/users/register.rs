use actix_web::{web, HttpResponse};
use sensitive::Sensitive;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Digest;
use sqlx::Row;
use validator::Validate;

use crate::{models::User, util::validation, App};

#[derive(Debug, Deserialize, Validate)]
pub struct PostRequest {
    #[validate(with = "validation::is_valid_username")]
    pub username: Sensitive<String>,
    #[validate(with = "validation::is_valid_email", optional)]
    pub email: Option<Sensitive<String>>,
    #[validate(length(min = 12, max = 128))]
    pub password: Sensitive<String>,
    #[validate(length(min = 12, max = 128))]
    pub confirm_password: Sensitive<String>,
}

#[derive(Debug, Serialize)]
pub struct PostResponse {
    pub verification_required: bool,
}

#[tracing::instrument]
pub async fn post(app: web::Data<App>, request: web::Json<PostRequest>) -> HttpResponse {
    request.validate().unwrap();

    // Check if that user exists, we need to append something
    // if user's email field is exists with this request data
    let mut query =
        sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT count(*) FROM \"users\" WHERE name = $1");

    if request.email.is_some() {
        query.push(" OR email = $2");
    }

    let mut query = query.build().bind(request.username.as_str());
    if let Some(email) = request.email.as_deref() {
        query = query.bind(email);
    }

    // We need to get the latest info as soon as possible because many
    // users will try to reserve their own user names.
    //
    // TODO: Add something like cache if possible
    let mut conn = app
        .db_write()
        .await
        .map_err(|e| {
            tracing::warn!(report = ?e, "Failed to get database connection");
            e
        })
        .unwrap();

    let amount = query
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| {
            tracing::warn!(report = ?e, "Failed to retrieve user data");
            e
        })
        .unwrap()
        .map(|v| v.get::<i64, _>("count"))
        .unwrap_or_default()
        .abs() as u64;

    if amount > 0 {
        return HttpResponse::BadRequest().json(json!({
            "message": "duplicated fields",
        }));
    }

    // TODO: Not secure right now but we need to get this asap
    let mut hasher = sha2::Sha512::default();
    hasher.update(format!("{}:{}", &*request.username, &*request.password));

    let password_hash = hex::encode(hasher.finalize());

    // Attempting to insert user right now!
    let _new_user = sqlx::query_as::<_, User>(
        r#"INSERT INTO "users" (name, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING *"#,
    )
    .bind(&*request.username)
    .bind(request.email.as_deref())
    .bind(password_hash)
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| {
        tracing::warn!(report = ?e, "Failed to insert user");
        e
    })
    .unwrap();

    HttpResponse::Created().json(PostResponse {
        verification_required: request.email.is_some(),
    })
}
