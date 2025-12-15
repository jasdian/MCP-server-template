mod common;

use common::*;
use mcp_server::auth::{AuthError, AuthLayer, AuthenticatedUser};
use mcp_server::ERROR_AUTH;
use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::Value;
use tower::{Layer, Service, ServiceExt};

// ============================================================================
// Mock Service for Testing Middleware
// ============================================================================

/// Mock service that returns 200 OK and checks for AuthenticatedUser extension
#[derive(Clone)]
struct MockService {
    /// Whether to check for AuthenticatedUser in extensions
    check_auth: bool,
}

impl MockService {
    fn new(check_auth: bool) -> Self {
        Self { check_auth }
    }
}

impl Service<Request> for MockService {
    type Response = Response;
    type Error = std::convert::Infallible;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let check_auth = self.check_auth;
        Box::pin(async move {
            if check_auth {
                // Verify AuthenticatedUser is in extensions
                let auth_user = req.extensions().get::<AuthenticatedUser>();
                assert!(auth_user.is_some(), "AuthenticatedUser not found in extensions");
            }
            Ok(StatusCode::OK.into_response())
        })
    }
}

// ============================================================================
// AuthError Response Tests
// ============================================================================

#[tokio::test]
async fn test_auth_error_missing_token_response() {
    let error = AuthError::MissingToken;
    let response = error.into_response();

    // Verify status code
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Extract body and parse JSON
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify JSON-RPC structure
    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["error"]["code"], ERROR_AUTH);
    assert_eq!(json["error"]["message"], "Missing Authorization header");
    assert!(json["error"]["data"].is_null());
}

#[tokio::test]
async fn test_auth_error_invalid_format_response() {
    let error = AuthError::InvalidFormat;
    let response = error.into_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["error"]["code"], ERROR_AUTH);
    assert_eq!(
        json["error"]["message"],
        "Invalid Authorization header format. Expected: Bearer <token>"
    );
}

#[tokio::test]
async fn test_auth_error_invalid_token_response() {
    let error = AuthError::InvalidToken;
    let response = error.into_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["error"]["code"], ERROR_AUTH);
    assert_eq!(json["error"]["message"], "Invalid or expired API key");
}

// ============================================================================
// AuthLayer Tests
// ============================================================================

#[test]
fn test_auth_layer_new() {
    let credentials = create_test_credentials_store();
    let layer = AuthLayer::new(credentials.clone());

    // Layer should be created successfully
    // We can't directly access the credentials field, but construction succeeds
    drop(layer);
}

#[test]
fn test_auth_layer_creates_middleware() {
    let credentials = create_test_credentials_store();
    let layer = AuthLayer::new(credentials);
    let mock_service = MockService::new(false);

    // Apply layer to create middleware
    let _middleware = layer.layer(mock_service);

    // Middleware created successfully
}

// ============================================================================
// Middleware Integration Tests
// ============================================================================

#[tokio::test]
async fn test_middleware_missing_authorization_header() {
    let credentials = create_test_credentials_store();
    let layer = AuthLayer::new(credentials);
    let mock_service = MockService::new(false);
    let mut service = layer.layer(mock_service);

    // Create request without Authorization header
    let request = Request::builder()
        .uri("/test")
        .body(Body::empty())
        .unwrap();

    // Call service
    let response = service.ready().await.unwrap().call(request).await.unwrap();

    // Verify 401 Unauthorized
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Verify JSON-RPC error
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["error"]["code"], ERROR_AUTH);
    assert_eq!(json["error"]["message"], "Missing Authorization header");
}

#[tokio::test]
async fn test_middleware_empty_authorization_header() {
    let credentials = create_test_credentials_store();
    let layer = AuthLayer::new(credentials);
    let mock_service = MockService::new(false);
    let mut service = layer.layer(mock_service);

    // Create request with empty Authorization header
    let request = Request::builder()
        .uri("/test")
        .header("Authorization", "")
        .body(Body::empty())
        .unwrap();

    let response = service.ready().await.unwrap().call(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["error"]["code"], ERROR_AUTH);
    assert_eq!(
        json["error"]["message"],
        "Invalid Authorization header format. Expected: Bearer <token>"
    );
}

#[tokio::test]
async fn test_middleware_wrong_auth_prefix() {
    let credentials = create_test_credentials_store();
    let layer = AuthLayer::new(credentials);
    let mock_service = MockService::new(false);
    let mut service = layer.layer(mock_service);

    // Create request with Basic auth instead of Bearer
    let request = Request::builder()
        .uri("/test")
        .header("Authorization", "Basic dGVzdDp0ZXN0")
        .body(Body::empty())
        .unwrap();

    let response = service.ready().await.unwrap().call(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["error"]["code"], ERROR_AUTH);
    assert_eq!(
        json["error"]["message"],
        "Invalid Authorization header format. Expected: Bearer <token>"
    );
}

#[tokio::test]
async fn test_middleware_no_bearer_prefix() {
    let credentials = create_test_credentials_store();
    let layer = AuthLayer::new(credentials);
    let mock_service = MockService::new(false);
    let mut service = layer.layer(mock_service);

    // Create request with token but no "Bearer " prefix
    let request = Request::builder()
        .uri("/test")
        .header("Authorization", "just-a-token")
        .body(Body::empty())
        .unwrap();

    let response = service.ready().await.unwrap().call(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["error"]["code"], ERROR_AUTH);
    assert_eq!(
        json["error"]["message"],
        "Invalid Authorization header format. Expected: Bearer <token>"
    );
}

#[tokio::test]
async fn test_middleware_invalid_api_key() {
    let credentials = create_test_credentials_store();
    let layer = AuthLayer::new(credentials);
    let mock_service = MockService::new(false);
    let mut service = layer.layer(mock_service);

    // Create request with invalid API key
    let request = Request::builder()
        .uri("/test")
        .header("Authorization", "Bearer invalid-api-key-xyz")
        .body(Body::empty())
        .unwrap();

    let response = service.ready().await.unwrap().call(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["error"]["code"], ERROR_AUTH);
    assert_eq!(json["error"]["message"], "Invalid or expired API key");
}

#[tokio::test]
async fn test_middleware_valid_api_key_injects_user() {
    let credentials = create_test_credentials_store();
    let layer = AuthLayer::new(credentials);
    let mock_service = MockService::new(true); // Check for AuthenticatedUser
    let mut service = layer.layer(mock_service);

    // Create request with valid API key
    let request = Request::builder()
        .uri("/test")
        .header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .body(Body::empty())
        .unwrap();

    let response = service.ready().await.unwrap().call(request).await.unwrap();

    // Should succeed (200 OK from mock service)
    assert_eq!(response.status(), StatusCode::OK);
    // Mock service internally verifies AuthenticatedUser is present
}

#[tokio::test]
async fn test_middleware_valid_api_key_proceeds_to_inner() {
    let credentials = create_test_credentials_store();
    let layer = AuthLayer::new(credentials);
    let mock_service = MockService::new(false);
    let mut service = layer.layer(mock_service);

    // Create request with valid API key
    let request = Request::builder()
        .uri("/test")
        .header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .body(Body::empty())
        .unwrap();

    let response = service.ready().await.unwrap().call(request).await.unwrap();

    // Should proceed to inner service and get 200 OK
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_middleware_multiple_requests_same_credentials() {
    let credentials = create_multi_user_credentials_store();
    let layer = AuthLayer::new(credentials);

    // First request with first user
    let mock_service1 = MockService::new(true);
    let mut service1 = layer.clone().layer(mock_service1);

    let request1 = Request::builder()
        .uri("/test1")
        .header("Authorization", format!("Bearer {}", TEST_API_KEY))
        .body(Body::empty())
        .unwrap();

    let response1 = service1.ready().await.unwrap().call(request1).await.unwrap();
    assert_eq!(response1.status(), StatusCode::OK);

    // Second request with second user
    let mock_service2 = MockService::new(true);
    let mut service2 = layer.layer(mock_service2);

    let request2 = Request::builder()
        .uri("/test2")
        .header("Authorization", format!("Bearer {}", TEST_API_KEY_2))
        .body(Body::empty())
        .unwrap();

    let response2 = service2.ready().await.unwrap().call(request2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_middleware_case_insensitive_authorization_header() {
    let credentials = create_test_credentials_store();
    let layer = AuthLayer::new(credentials);
    let mock_service = MockService::new(true);
    let mut service = layer.layer(mock_service);

    // HTTP headers are case-insensitive, test with lowercase
    let request = Request::builder()
        .uri("/test")
        .header("authorization", format!("Bearer {}", TEST_API_KEY))
        .body(Body::empty())
        .unwrap();

    let response = service.ready().await.unwrap().call(request).await.unwrap();

    // Should succeed
    assert_eq!(response.status(), StatusCode::OK);
}
