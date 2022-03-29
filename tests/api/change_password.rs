use crate::helpers::{spawn_app, assert_is_redirect_to};
use fake::faker::internet::en::Password;
use fake::Fake;
use serde_json::Value;
use uuid::Uuid;

#[actix_rt::test]
async fn you_must_be_logged_in_to_see_the_change_password_form() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.get_change_password().await;

    // Assert
    assert_is_redirect_to(&response,"/login");
}

#[actix_rt::test]
async fn you_must_be_logged_in_to_change_your_password() {
    // Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    // Act
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": Uuid::new_v4().to_string(),
            "new_password": &new_password,
            "new_password_check": &new_password,
        }))
        .await;

    // Assert
    assert_is_redirect_to(&response,"/login");
}

#[actix_rt::test]
async fn new_password_fields_must_match() {
    // Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let another_new_password = Uuid::new_v4().to_string();

    // Act - Login
    app.test_user.login(&app).await;

    // Act - Try to change password
    let response = app
        .post_change_password(&create_change_password_body(
            &app.test_user.password,
            &new_password,
            &another_new_password,
        ))
        .await;
    assert_is_redirect_to(&response,"/admin/password");

    // Act - Follow the redirect
    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains(
        "<p><i>You entered two different new passwords - the field values must match.</i></p>"
    ));
}

#[actix_rt::test]
async fn current_password_must_be_valid() {
    // Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let wrong_password = Uuid::new_v4().to_string();

    // Act - Login
    app.test_user.login(&app).await;

    // Act - Try to change password
    let response = app
        .post_change_password(&create_change_password_body(
            &wrong_password,
            &new_password,
            &new_password,
        ))
        .await;
    assert_is_redirect_to(&response,"/admin/password");

    // Act - Follow the redirect
    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>The current password is incorrect.</i></p>"));
}

#[actix_rt::test]
async fn current_password_must_be_correct_length() {
    let app = spawn_app().await;

    let test_cases: Vec<(String, &str)> = vec![
        (
            Password(1..11).fake(),
            "The new password must be at least 12 characters.",
        ),
        (
            Password(128..200).fake(),
            "The new password must be shorter than 128 characters.",
        ),
    ];

    for (invalid_password, error_message) in test_cases {
        // Act - Login
        app.test_user.login(&app).await;

        let invalid_password = invalid_password.as_str();

        // Act - Try to change password
        let response = app
            .post_change_password(&create_change_password_body(
                &app.test_user.password,
                invalid_password,
                invalid_password,
            ))
            .await;
        assert_is_redirect_to(&response,"/admin/password");

        // Act - Follow the redirect
        let html_page = app.get_change_password_html().await;
        assert!(html_page.contains(format!("<p><i>{}</i></p>", &error_message).as_str()));
    }
}

#[actix_rt::test]
async fn changing_password_works() {
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    // Act - Login
    let response = app.test_user.login(&app).await;
    assert_is_redirect_to(&response,"/admin/dashboard");

    // Act - Change password
    let response = app
        .post_change_password(&create_change_password_body(
            &app.test_user.password,
            &new_password,
            &new_password,
        ))
        .await;
    assert_is_redirect_to(&response,"/admin/password");

    // Act - Follow the redirect
    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>Your password has been changed.</i></p>"));

    // Act - Logout
    let response = app.post_logout().await;
    assert_is_redirect_to(&response,"/login");

    // Act - Follow the redirect
    let html_page = app.get_login_html().await;
    assert!(html_page.contains(r#"<p><i>You have successfully logged out.</i></p>"#));

    // Act - Login using the new password
    let response = app
        .post_login(&serde_json::json!({
            "username": &app.test_user.username,
            "password": &new_password
        }))
        .await;
    assert_is_redirect_to(&response,"/admin/dashboard");
}

fn create_change_password_body(
    current_password: &str,
    new_password: &str,
    new_password_check: &str,
) -> Value {
    serde_json::json!({
        "current_password": current_password,
        "new_password": new_password,
        "new_password_check": new_password_check,
    })
}
