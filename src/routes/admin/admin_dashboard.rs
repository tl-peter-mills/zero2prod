use actix_web::{web, HttpResponse};
use crate::session_state::TypedSession;
use uuid::Uuid;
use sqlx::PgPool;
use actix_web::http::header::ContentType;
use anyhow::Context;
use crate::utils::{e500, see_login};

pub async fn admin_dashboard(
    session: TypedSession,
    pool: web::Data<PgPool>
) -> Result<HttpResponse, actix_web::Error> {
    let username = if let Some(user_id) = session.get_user_id().map_err(e500)? {
        get_username(user_id, &pool).await.map_err(e500)?
    } else {
        return Ok(see_login());
    };
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Admin dashboard</title>
</head>
<body>
    <p>Welcome {}!</p>
    <p>Available actions:</p>
    <ol>
        <li><a href="/admin/password">Change password</a></li>
        <li>
            <a href="javascript:document.logoutForm.submit()">Logout</a>
            <form name="logoutForm" action="/admin/logout" method="post" hidden>
                <input hidden type="submit" value="Logout">
            </form>
        </li>
    </ol>
</body>
</html>"#,
            username
        )))
}

#[tracing::instrument(name = "Get username", skip(pool))]
pub async fn get_username(
    user_id: Uuid,
    pool: &PgPool
) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT username
        FROM users
        WHERE user_id = $1
        "#,
        user_id
    )
        .fetch_one(pool)
        .await
        .context("A database error was encountered while trying to get a username.")?;
    Ok(row.username)
}


