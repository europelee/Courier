use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use serde_json::{json, Value};
use http_body_util::BodyExt;

async fn make_app(password: &str) -> axum::Router {
    courier_server::test_helpers::make_test_router(password).await
}

#[tokio::test]
async fn test_login_correct_password() {
    let app = make_app("mysecret").await;
    let body = serde_json::to_vec(&json!({"password": "mysecret"})).unwrap();
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json["token"].as_str().is_some());
    assert_eq!(json["expires_in"], 86400);
}

#[tokio::test]
async fn test_login_wrong_password() {
    let app = make_app("mysecret").await;
    let body = serde_json::to_vec(&json!({"password": "wrong"})).unwrap();
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_missing_password_field() {
    let app = make_app("mysecret").await;
    let body = serde_json::to_vec(&json!({})).unwrap();
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
