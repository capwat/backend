use actix_web::{http::header, web, FromRequest, HttpResponse};
use chrono::{NaiveDateTime, Utc};
use futures_util::Future;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::pin::Pin;

use crate::{
    models::{id::UserId, User},
    App,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Jwt {
    pub created_at: NaiveDateTime,
    pub issuer: String,
    pub exp_secs_until: u64,
    pub user_id: UserId,
}

impl Jwt {
    pub async fn get_user(&self, app: &App) -> Result<User, HttpResponse> {
        let mut conn = app
            .db_read()
            .await
            .map_err(|e| {
                tracing::warn!(report = ?e, "Failed to get database connection");
                e
            })
            .unwrap();

        let user = sqlx::query_as::<_, User>(r#"SELECT * FROM "users" WHERE id = $1"#)
            .bind(&self.user_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| {
                tracing::warn!(report = ?e, "Failed to retrieve user data");
                e
            })
            .unwrap();

        if let Some(user) = user {
            Ok(user)
        } else {
            Err(HttpResponse::Unauthorized().json(json!({
                "message": "Authentication required",
            })))
        }
    }
}

impl FromRequest for Jwt {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let token = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        let token = match token {
            Some(n) => n,
            None => {
                return Box::pin(async {
                    Err(actix_web::error::ErrorUnauthorized(json!({
                        "message": "Authentication required",
                    })))
                })
            }
        };

        let app = req
            .app_data::<web::Data<App>>()
            .expect("web::Data<App> is missing")
            .clone();

        let jwt = Jwt::decode(token, &app);
        Box::pin(async move { Ok(jwt) })
    }
}

impl Jwt {
    #[tracing::instrument(skip(token))]
    pub fn decode(token: &str, app: &App) -> Self {
        let key = DecodingKey::from_secret(app.config.jwt_secret.as_str().as_bytes());
        let mut validation = Validation::new(Algorithm::HS512);
        validation.validate_exp = false;
        validation.required_spec_claims = Default::default();

        jsonwebtoken::decode::<Self>(&token, &key, &validation)
            .expect("failed to decode jwt")
            .claims
    }

    #[tracing::instrument(skip(user_id))]
    pub async fn encode(user_id: UserId, app: web::Data<App>) -> String {
        tokio::task::spawn_blocking(move || {
            let header = Header {
                alg: Algorithm::HS512,
                ..Default::default()
            };
            let claims = Self {
                created_at: Utc::now().naive_utc(),
                issuer: "server".into(),
                exp_secs_until: 1000000,
                user_id,
            };
            let key = EncodingKey::from_secret(app.config.jwt_secret.as_str().as_bytes());
            jsonwebtoken::encode(&header, &claims, &key).expect("failed to create jwt")
        })
        .await
        .unwrap()
    }
}
