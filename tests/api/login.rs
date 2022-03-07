use crate::helpers::spawn_app;

#[actix_rt::test]
async fn an_error_flash_message_is_set_on_failure() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to login
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password"
    });
    let response = app.post_login(&login_body).await;

    // Assert
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("Location").unwrap(), "/login");

    // Act - Follow the redirect
    let response = app.get_login().await;
    let html_page = response.text().await.unwrap();

    let expected_error_html = r#"<p><i>Authentication failed</i></p>"#;

    assert!(html_page.contains(expected_error_html));

    // Act - Refresh the login page
    let response = app.get_login().await;
    let html_page = response.text().await.unwrap();

    assert!(!html_page.contains(expected_error_html));
}
