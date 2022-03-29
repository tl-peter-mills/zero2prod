use crate::helpers::{spawn_app, assert_is_redirect_to};

#[actix_rt::test]
async fn you_must_be_logged_in_to_access_the_admin_dashboard() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.get_admin_dashboard().await;

    // Assert
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("Location").unwrap(), "/login");
}

#[actix_rt::test]
async fn logout_clears_session_state() {
    // Arrange
    let app = spawn_app().await;

    // Act - Login
    let response = app.test_user.login(&app).await;
    assert_is_redirect_to(&response,"/admin/dashboard");

    // Act - Follow the redirect
    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome {}", app.test_user.username)));

    // Act - Logout
    let response = app.post_logout().await;
    assert_is_redirect_to(&response,"/login");

    // Act - Follow the redirect
    let html_page = app.get_login_html().await;
    assert!(html_page.contains(r#"<p><i>You have successfully logged out.</i></p>"#));

    // Act - Attempt to load admin panel
    let response = app.get_admin_dashboard().await;
    assert_is_redirect_to(&response,"/login");
}
