use super::error::AuthError;
use super::types::{AuthenticatedUser, CredentialsStore, validate_api_key};
use axum::{
    extract::Request,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// Tower Layer for authentication
/// Creates AuthMiddleware instances
#[derive(Clone)]
pub struct AuthLayer {
    credentials: CredentialsStore,
}

impl AuthLayer {
    /// Create a new authentication layer
    pub fn new(credentials: CredentialsStore) -> Self {
        Self { credentials }
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware {
            inner,
            credentials: self.credentials.clone(),
        }
    }
}

/// Tower Service for authentication
/// Validates API key and injects authenticated user into request extensions
#[derive(Clone)]
pub struct AuthMiddleware<S> {
    inner: S,
    credentials: CredentialsStore,
}

impl<S> Service<Request> for AuthMiddleware<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        let credentials = self.credentials.clone();
        let auth_result = authenticate(req.headers(), &credentials);

        match auth_result {
            Ok(user_credentials) => {
                // Inject authenticated user into request extensions
                req.extensions_mut()
                    .insert(AuthenticatedUser(user_credentials));

                // Forward to inner service
                let future = self.inner.call(req);
                Box::pin(future)
            }
            Err(auth_error) => {
                // Return 401 Unauthorized
                Box::pin(async move { Ok(auth_error.into_response()) })
            }
        }
    }
}

/// Extract and validate Bearer token from request headers
fn authenticate(
    headers: &HeaderMap,
    credentials: &CredentialsStore,
) -> Result<super::types::UserCredentials, AuthError> {
    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(AuthError::MissingToken)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidFormat)?;

    validate_api_key(token, credentials).ok_or(AuthError::InvalidToken)
}
