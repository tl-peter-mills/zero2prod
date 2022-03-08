use actix_web::{HttpResponse, web};
use secrecy::{Secret, ExposeSecret};
use crate::session_state::TypedSession;
use crate::utils::{e500, see_login, see_other};
use actix_web_flash_messages::FlashMessage;
use sqlx::PgPool;
use crate::routes::admin::admin_dashboard::get_username;
use crate::authentication::{Credentials, validate_credentials, AuthError};
use validator::HasLen;

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(
    form: web::Form<FormData>,
    session: TypedSession,
    pool: web::Data<PgPool>
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = session.get_user_id().map_err(e500)?;
    if user_id.is_none() {
        return Ok(see_login());
    }
    let user_id = user_id.unwrap();
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        FlashMessage::error(
            "You entered two different new passwords - the field values must match.",
        )
        .send();
        return Ok(see_admin_password());
    }
    let username = get_username(user_id, &pool).await.map_err(e500)?;
    let credentials = Credentials {
        username,
        password: form.0.current_password,
    };
    if let Err(e) = validate_credentials(credentials, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect.").send();
                Ok(see_admin_password())
            }
            AuthError::UnexpectedError(_) => Err(e500(e).into()),
        }
    }
    if let Err(e) = validate_password(&form.0.new_password) {
        FlashMessage::error(e.to_string()).send();
        return Ok(see_admin_password());
    }
    crate::authentication::change_password(user_id, form.0.new_password, &pool)
        .await.map_err(e500)?;
    FlashMessage::info("Your password has been changed.").send();
    Ok(see_other("/admin/password"))
}

fn see_admin_password() -> HttpResponse {
    see_other("/admin/password")
}

fn validate_password(password: &Secret<String>) -> Result<(), anyhow::Error> {
    if password.expose_secret().length() < 12 {
        return Err(anyhow::anyhow!("The new password must be at least 12 characters."));
    }

    if password.expose_secret().length() > 128 {
        return Err(anyhow::anyhow!("The new password must be shorter than 128 characters."));
    }

    Ok(())
}