use crate::helpers::{assert_is_redirect_to, spawn_app, ConfirmationLinks, TestApp};
use std::time::Duration;
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=le%20guin&email=ursula_le_guin%40yopmail.com";
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_links(&email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Act - Part 1 - Submit newsletter form
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_publish_newsletter_html().await;
    assert!(html_page.contains("<p><i>The newsletter issue has been published!</i></p>"));
    // Mock verifies on Drop that we haven't sent the newsletter email
}

// #[tokio::test]
// async fn newsletters_are_delivered_to_confirmed_subscribers() {
//     let app = spawn_app().await;
//     create_confirmed_subscriber(&app).await;
//     app.test_user.login(&app).await;
//
//     Mock::given(path("/email"))
//         .and(method("POST"))
//         .respond_with(ResponseTemplate::new(200))
//         .expect(1)
//         .mount(&app.email_server)
//         .await;
//
//     let newsletter_request_body = serde_json::json!({
//         "title": "Newsletter title",
//         "text_content": "Newsletter body as plain text",
//         "html_content": "<p>Newsletter body as HTML</p>"
//     });
//     let response = app.post_publish_newsletter(&newsletter_request_body).await;
//     assert_is_redirect_to(&response, "/admin/newsletters");
//
//     let html_page = app.get_publish_newsletter_html().await;
//     assert!(html_page.contains("<p><i>The newsletter issue has been published!</i></p>"));
// }

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - Part 1 - Submit newsletter form
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_publish_newsletter_html().await;
    assert!(html_page.contains("<p><i>The newsletter issue has been published!</i></p>"));
    // Mock verifies on Drop that we have sent the newsletter email
}

// #[tokio::test]
// async fn newsletters_returns_400_for_invalid_data() {
//     let app = spawn_app().await;
//     let combinations = vec![
//         (
//             serde_json::json!({
//                 "content": {
//                     "text": "Newsletter body as plain text",
//                     "html": "<p>Newsletter body as HTML</p>",
//                 }
//             }),
//             "missing title",
//         ),
//         (
//             serde_json::json!({"title": "Newsletter bruh!"}),
//             "missing content",
//         ),
//     ];
//
//     for (body, msg) in combinations {
//         let response = app.post_newsletters(body).await;
//
//         assert_eq!(
//             response.status().as_u16(),
//             400,
//             "The API did not fail with 400 Bad Request when the payload was {}.",
//             msg
//         );
//     }
// }

// #[tokio::test]
// async fn requests_missing_authorization_are_rejected() {
//     let app = spawn_app().await;
//
//     let response = reqwest::Client::new()
//         .post(&format!("{}/newsletters", &app.address))
//         .json(&serde_json::json!({
//             "title": "Newsletter title",
//             "content": {
//                 "text": "Newsletter bobdy as plain text",
//                 "html": "<p>Newsletter bobdy as plain text</p>",
//             }
//         }))
//         .send()
//         .await
//         .expect("Failed to execute request.");
//
//     assert_eq!(401, response.status().as_u16());
//     assert_eq!(
//         r#"Basic realm="publish""#,
//         response.headers()["WWW-Authenticate"]
//     );
// }

#[tokio::test]
async fn you_must_be_logged_in_to_see_the_newsletter_form() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.get_publish_newsletter().await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_publish_a_newsletter() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response = app.post_publish_newsletter(&newsletter_request_body).await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = app.get_publish_newsletter_html().await;
    assert!(dbg!(html_page).contains("<p><i>The newsletter issue has been published!</i></p>"));

    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = app.get_publish_newsletter_html().await;
    assert!(dbg!(html_page).contains("<p><i>The newsletter issue has been published!</i></p>"));
}

#[tokio::test]
async fn concurrenet_form_submission_is_handled_gracefully() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response1 = app.post_publish_newsletter(&newsletter_request_body);
    let response2 = app.post_publish_newsletter(&newsletter_request_body);
    let (response1, response2) = tokio::join!(response1, response2);
    assert_eq!(response1.status(), response2.status());
    assert_eq!(response1.text().await.unwrap(), response2.text().await.unwrap());
}
