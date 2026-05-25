//! Shared Axum extractors for authenticated request handling.
//!
//! Authentication remains backed by the existing opaque token service. This
//! module adapts that service into an extractor so successful authentication can
//! also annotate the request-scoped API usage context with the trusted user id.

use std::ops::Deref;

use async_trait::async_trait;
use axum::{extract::FromRequestParts, http::request::Parts};
use stellartrail_db::repositories::UserRecord;

use crate::{
    api_usage::ApiUsageUserContext, error::ApiError, services::auth_service, state::AppState,
};

/// Authenticated current user extracted from a Bearer access token.
#[derive(Clone, Debug)]
pub struct AuthenticatedUser(pub UserRecord);

impl Deref for AuthenticatedUser {
    type Target = UserRecord;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = auth_service::authenticate(&parts.headers, state).await?;
        if let Some(context) = parts.extensions.get::<ApiUsageUserContext>() {
            context.set_user_id(user.id.clone());
        }
        Ok(Self(user))
    }
}
