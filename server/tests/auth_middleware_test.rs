use axum::{
    body::Body,
    http::{Request, StatusCode, header::AUTHORIZATION},
    routing::get,
    Router,
    middleware,
};
use tower::ServiceExt;

async fn dummy_handler() -> &'static str { "ok" }

async fn make_test_app(password: &str) -> Router {
    let state = courier_server::test_helpers::make_app_state(password).await;
    Router::new()
        .route("/protected", get(dummy_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            courier_server::auth::auth_middleware,
        ))
        .with_state(state)
}

#[tokio::test]
async fn test_no_token_returns_401() {
    let app = make_test_app("test_secret").await;
    let resp = app
        .oneshot(Request::builder().uri("/protected").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_valid_token_passes() {
    let secret = "test_secret";
    let app = make_test_app(secret).await;
    let token = courier_server::auth::generate_token("admin".to_string(), 24, secret).unwrap();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_invalid_token_returns_401() {
    let app = make_test_app("test_secret").await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .header(AUTHORIZATION, "Bearer invalid.token.here")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
