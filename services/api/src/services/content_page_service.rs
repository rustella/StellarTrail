//! Content page service for validating public selectors and parsing stored JSON.

use stellartrail_db::repositories::AppContentPageRepository;
use stellartrail_domain::validation::FieldViolation;

use crate::{
    dto::content_page::{ContentPageContent, ContentPageQuery, ContentPageResponse},
    error::ApiError,
    state::AppState,
};

const CLIENT_KEYS: [&str; 5] = ["wechat_miniprogram", "web", "android", "ios", "macos"];
const DEFAULT_CLIENT_KEY: &str = "wechat_miniprogram";
const DEFAULT_LOCALE: &str = "zh-CN";

/// Reads one published content page for a public client.
pub async fn get_public(
    state: &AppState,
    page_key: String,
    query: ContentPageQuery,
) -> Result<ContentPageResponse, ApiError> {
    let page_key = normalize_page_key(page_key)?;
    let client_key = normalize_client_key(query.client_key.unwrap_or_default())?;
    let locale = normalize_locale(query.locale.unwrap_or_default())?;
    let record = AppContentPageRepository::new(state.db().clone())
        .get_published(&page_key, &client_key, &locale)
        .await?
        .ok_or(ApiError::NotFound)?;
    let content = serde_json::from_str::<ContentPageContent>(&record.content_json)
        .map_err(ApiError::internal)?;
    Ok(ContentPageResponse::from_record(&record, content))
}

fn normalize_page_key(value: String) -> Result<String, ApiError> {
    let page_key = value.trim().to_ascii_lowercase();
    if page_key.is_empty()
        || page_key.len() > 80
        || !page_key
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "page_key",
            "is not supported",
        )]));
    }
    Ok(page_key)
}

fn normalize_client_key(value: String) -> Result<String, ApiError> {
    let client_key = if value.trim().is_empty() {
        DEFAULT_CLIENT_KEY.to_owned()
    } else {
        value.trim().to_ascii_lowercase()
    };
    if !CLIENT_KEYS.contains(&client_key.as_str()) {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "client_key",
            "is not supported",
        )]));
    }
    Ok(client_key)
}

fn normalize_locale(value: String) -> Result<String, ApiError> {
    let locale = value.trim();
    if locale.is_empty() || locale.eq_ignore_ascii_case(DEFAULT_LOCALE) {
        return Ok(DEFAULT_LOCALE.to_owned());
    }
    Err(ApiError::Validation(vec![FieldViolation::new(
        "locale",
        "is not supported",
    )]))
}
