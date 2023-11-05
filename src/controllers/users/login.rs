use actix_web::{web, HttpResponse};
use sensitive::Sensitive;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Digest;
use validator::Validate;

use crate::{
    auth::Jwt,
    models::{id::UserId, User},
    App,
};

#[derive(Debug, Deserialize, Validate)]
pub struct PostRequest {
    #[validate(length(min = 1))]
    pub username_or_email: Sensitive<String>,
    #[validate(length(min = 1))]
    pub password: Sensitive<String>,
}

#[derive(Debug, Serialize)]
pub struct PostResponse {
    pub id: UserId,
    pub token: String,
}

#[tracing::instrument]
pub async fn post(app: web::Data<App>, request: web::Json<PostRequest>) -> HttpResponse {
    request.validate().unwrap();

    // We need to get the latest info as soon as possible
    let mut conn = app
        .db_read_prefer_primary()
        .await
        .map_err(|e| {
            tracing::warn!(report = ?e, "Failed to get database connection");
            e
        })
        .unwrap();

    let user = sqlx::query_as::<_, User>(r#"SELECT * FROM "users" WHERE name = $1 OR email = $1"#)
        .bind(request.username_or_email.as_str())
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| {
            tracing::warn!(report = ?e, "Failed to retrieve user data");
            e
        })
        .unwrap();

    let user = match user {
        Some(n) => n,
        None => {
            return HttpResponse::Forbidden().json(json!({
                "message": "Invalid credientials",
            }))
        }
    };

    // TODO: Not secure right now but we need to get this asap
    let mut hasher = sha2::Sha512::default();
    hasher.update(format!("{}:{}", user.name, &*request.password));

    let attempt_password_hash = hex::encode(hasher.finalize());

    let mut matched = true;
    for (a, b) in user
        .password_hash
        .chars()
        .zip(attempt_password_hash.chars())
    {
        matched = matched && (a == b);
    }

    if !matched {
        return HttpResponse::Forbidden().json(json!({
            "message": "Invalid credientials",
        }));
    }

    drop(conn);

    let jwt = Jwt::encode(user.id, app).await;
    HttpResponse::Ok().json(PostResponse {
        id: user.id,
        token: jwt,
    })
}
