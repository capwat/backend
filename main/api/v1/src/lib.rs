use actix_web::web;
use api_common::Context;
use capwat_kernel::{domain::users, Result};
use std::sync::Arc;

pub async fn register(
    ctx: web::Data<Arc<Context>>,
    form: web::Json<capwat_types_v1::users::Register>,
) -> Result<capwat_types_v1::users::Register> {
    let form = users::CreateUser {
        name: form.username.as_deref(),
        email: form.email.as_opt_deref(),
        password_hash: form.password.as_deref(),
    };
    let user = ctx.users.create(form).await?;

    Ok(capwat_types_v1::users::Register {
        username: "i".into(),
        email: None.into(),
        password: "Hello".into(),
        confirm_password: "Hello".into(),
    })
}
