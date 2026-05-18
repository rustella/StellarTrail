//! Administrator routes for uploading Knots3D media into object storage and DB metadata.

use axum::{
    Json, Router,
    extract::{Multipart, Path, State},
    http::StatusCode,
    routing::put,
};
use stellartrail_domain::validation::FieldViolation;

use crate::{
    dto::knot_media::KnotMediaUploadResponse, error::ApiError, extractors::AuthenticatedUser,
    services::knot_media_upload_service, state::AppState,
};

/// Builds administrator-only knot media routes.
pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/api/admin/skills/knots/:knot_id/media/:asset_id",
        put(upload_knot_media),
    )
}

async fn upload_knot_media(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path((knot_id, asset_id)): Path<(String, String)>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<KnotMediaUploadResponse>), ApiError> {
    knot_media_upload_service::ensure_admin(&user, &state)?;
    let input = parse_upload_multipart(multipart).await?;
    let response =
        knot_media_upload_service::upload_knot_media(&state, &user, &knot_id, &asset_id, input)
            .await?;
    Ok((StatusCode::CREATED, Json(response)))
}

async fn parse_upload_multipart(
    mut multipart: Multipart,
) -> Result<knot_media_upload_service::KnotMediaUploadInput, ApiError> {
    let mut media_type = None;
    let mut file_name = None;
    let mut content_type = None;
    let mut bytes = None;
    let mut attribution = None;
    let mut license_note = None;
    let mut source_name = None;
    let mut source_path = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| ApiError::BadRequest(format!("invalid multipart body: {error}")))?
    {
        let name = field.name().map(ToOwned::to_owned);
        match name.as_deref() {
            Some("file") => {
                file_name = field.file_name().map(ToOwned::to_owned);
                content_type = field.content_type().map(ToOwned::to_owned);
                let field_bytes = field.bytes().await.map_err(|error| {
                    ApiError::BadRequest(format!("invalid multipart file: {error}"))
                })?;
                bytes = Some(field_bytes.to_vec());
            }
            Some("media_type") => media_type = Some(read_text_field(field).await?),
            Some("attribution") => attribution = non_empty(read_text_field(field).await?),
            Some("license_note") => license_note = non_empty(read_text_field(field).await?),
            Some("source_name") => source_name = non_empty(read_text_field(field).await?),
            Some("source_path") => source_path = non_empty(read_text_field(field).await?),
            _ => {
                // Ignore unknown fields so the upload CLI can add future non-sensitive hints without breaking old servers.
            }
        }
    }

    let media_type = media_type.ok_or_else(|| {
        ApiError::Validation(vec![FieldViolation::new("media_type", "is required")])
    })?;
    let bytes = bytes
        .ok_or_else(|| ApiError::Validation(vec![FieldViolation::new("file", "is required")]))?;
    Ok(knot_media_upload_service::KnotMediaUploadInput {
        media_type,
        original_filename: file_name,
        declared_content_type: content_type,
        bytes,
        attribution,
        license_note,
        source_name,
        source_path,
    })
}

async fn read_text_field(field: axum::extract::multipart::Field<'_>) -> Result<String, ApiError> {
    field
        .text()
        .await
        .map(|value| value.trim().to_owned())
        .map_err(|error| ApiError::BadRequest(format!("invalid multipart field: {error}")))
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}
