use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_health_no_token_still_200() {
    let app = courier_server::test_helpers::make_test_router("secret").await;
    let resp = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[test]
fn test_validate_auth_token_rejects_empty_string() {
    // When no token is provided (empty string simulates missing token from ws_msg.data),
    // validate_auth_token should return Err, causing connection to be rejected
    let result = courier_server::auth::validate_auth_token("", "test_secret");
    assert!(result.is_err(), "missing token should be rejected");
}

#[test]
fn test_subscribe_token_validation_accepts_valid_token() {
    // When a valid token is provided, validate_auth_token should return Ok,
    // allowing the connection to proceed to handle_subscriber_connection
    let secret = "test_secret";
    let token = courier_server::auth::generate_token("admin".to_string(), 24, secret)
        .expect("token generation should succeed");
    let result = courier_server::auth::validate_auth_token(&token, secret);
    assert!(result.is_ok(), "valid token should be accepted");
}
