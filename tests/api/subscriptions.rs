use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use rand::Rng;

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let test_app = spawn_app().await;
    let request_body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    // Act
    let response = test_app.post_subscriptions(request_body.into()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[actix_rt::test]
async fn subscribe_persists_the_new_subscriber() {
    // Arrange
    let test_app = spawn_app().await;
    let request_body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    // Act
    test_app.post_subscriptions(request_body.into()).await;

    // Assert
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "pending_confirmation");
}

#[actix_rt::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let test_app = spawn_app().await;
    let test_cases = vec![
        ("name=le%20quin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both email and name"),
    ];

    for (invalid_body, test_failure_message) in test_cases {
        // Act
        let response = test_app.post_subscriptions(invalid_body.into()).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            test_failure_message
        )
    }
}

#[actix_rt::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_empty() {
    // Arrange
    let test_app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        // Act
        let response = test_app.post_subscriptions(body.into()).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 200 OK when the payload was {}.",
            description
        )
    }
}

#[actix_rt::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    // Arrange
    let test_app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    // Act
    test_app.post_subscriptions(body.into()).await;
}

#[actix_rt::test]
async fn subscribing_multiple_times_sends_multiple_confirmation_emails() {
    // Arrange
    let test_app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let times_clicked = rand::thread_rng().gen_range(2..10);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(times_clicked)
        .mount(&test_app.email_server)
        .await;

    // Act
    for _ in 0..times_clicked {
        test_app.post_subscriptions(body.into()).await;
    }
}

#[actix_rt::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    // Arrange
    let test_app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    // Act
    test_app.post_subscriptions(body.into()).await;

    // Assert
    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = test_app.get_confirmation_links(&email_request);

    assert_eq!(confirmation_links.html, confirmation_links.plain_text);
}

#[actix_rt::test]
async fn subscribing_multiple_times_sends_the_same_confirmation_link_each_time() {
    // Arrange
    let test_app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let times_clicked = rand::thread_rng().gen_range(2..10);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(times_clicked)
        .mount(&test_app.email_server)
        .await;

    // Act
    for _ in 0..times_clicked {
        test_app.post_subscriptions(body.into()).await;
    }

    // Assert
    let first_request = &test_app.email_server.received_requests().await.unwrap()[0];
    let first_confirmation_links = test_app.get_confirmation_links(&first_request);

    for i in 1..times_clicked {
        let email_request = &test_app.email_server.received_requests().await.unwrap()[i as usize];
        let confirmation_links = test_app.get_confirmation_links(&email_request);

        assert_eq!(first_confirmation_links.html, confirmation_links.html);
        assert_eq!(first_confirmation_links.plain_text, confirmation_links.plain_text);
    }
}
