use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_200_with_valid_form_data() {
    // arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40yopmail.com";

    // act
    let response = app.post_subscriptions(body.into()).await;

    // assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@yopmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_with_missing_data() {
    // arrange
    let app = spawn_app().await;

    // parametrised test
    let combinations = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40yopmail.com", "missing the name"),
        ("", "missing email and name"),
    ];

    for (invalid_body, error_msg) in combinations {
        // act
        let response = app.post_subscriptions(invalid_body.into()).await;

        // assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // additioinal error on test failure
            "The API did not failed with 400 Bad Request: payload was {:?}",
            error_msg
        );
    }
}

#[tokio::test]
async fn subscribe_returns_400_when_fields_are_present_but_invalid() {
    let app = spawn_app().await;
    let combinations = vec![
        ("name=&email=ursula_le_guin%40.yopmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=not-an-email", "invalid email"),
    ];

    for (body, description) in combinations {
        let response = app.post_subscriptions(body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            // additioinal error on test failure
            "The API did not return 400 OK when the payload was {:?}",
            description
        )
    }
}
