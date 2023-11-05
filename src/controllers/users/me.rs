use actix_web::{web, HttpResponse};
use chrono::NaiveDateTime;
use serde::Serialize;

use crate::{auth::Jwt, models::id::UserId, App};

#[derive(Debug, Serialize)]
pub struct GetResponse {
    pub id: UserId,
    pub created_at: NaiveDateTime,
    pub name: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
}

#[tracing::instrument(skip(jwt))]
pub async fn get(app: web::Data<App>, jwt: Jwt) -> HttpResponse {
    let user = match jwt.get_user(&app).await {
        Ok(n) => n,
        Err(e) => return e,
    };
    HttpResponse::Ok().json(GetResponse {
        id: user.id,
        created_at: user.created_at,
        name: user.name,
        display_name: user.display_name,
        email: user.email,
    })
}
