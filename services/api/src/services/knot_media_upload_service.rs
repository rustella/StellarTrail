//! Service for administrator-managed Knots3D media uploads backed by object storage and DB metadata.

use std::collections::HashMap;

use sha2::{Digest, Sha256};
use stellartrail_db::repositories::{
    KnotMediaLinkDraft, MediaResourceDraft, MediaResourceRepository, UserRecord,
};
use stellartrail_domain::{skill::KnotMediaAsset, validation::FieldViolation};

use crate::{
    dto::knot_media::KnotMediaUploadResponse, error::ApiError, object_store::PutObjectRequest,
    state::AppState,
};

const PROVIDER_MINIO: &str = "minio";
const CACHE_CONTROL_IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// Parsed multipart upload payload for one knot media asset.
#[derive(Debug)]
pub struct KnotMediaUploadInput {
    pub media_type: String,
    pub original_filename: Option<String>,
    pub declared_content_type: Option<String>,
    pub bytes: Vec<u8>,
    pub attribution: Option<String>,
    pub license_note: Option<String>,
    pub source_name: Option<String>,
    pub source_path: Option<String>,
}

/// Authenticates administrator intent against the configured allowlist.
pub fn ensure_admin(user: &UserRecord, state: &AppState) -> Result<(), ApiError> {
    let admin = &state.config().admin;
    let username = user.username.as_deref().map(str::to_ascii_lowercase);
    let email = user.email.as_deref().map(str::to_ascii_lowercase);
    let allowed = admin.user_ids.iter().any(|id| id == &user.id)
        || username
            .as_deref()
            .is_some_and(|value| admin.usernames.iter().any(|item| item == value))
        || email
            .as_deref()
            .is_some_and(|value| admin.emails.iter().any(|item| item == value));
    if allowed {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

/// Handles validation, object write, DB metadata upsert, and public response construction.
pub async fn upload_knot_media(
    state: &AppState,
    user: &UserRecord,
    knot_id: &str,
    asset_id: &str,
    input: KnotMediaUploadInput,
) -> Result<KnotMediaUploadResponse, ApiError> {
    validate_safe_slug("knot_id", knot_id, 96)?;
    validate_safe_slug("asset_id", asset_id, 64)?;
    let spec = asset_spec(asset_id).ok_or_else(|| {
        ApiError::Validation(vec![FieldViolation::new(
            "asset_id",
            "must be one of thumbnail, preview, draw_gif, turntable_gif, draw_mp4, turntable_mp4",
        )])
    })?;
    if input.media_type != spec.media_type {
        return Err(ApiError::BadRequest(format!(
            "media_type `{}` does not match asset `{asset_id}`",
            input.media_type
        )));
    }
    let repo = MediaResourceRepository::new(state.db().clone());
    if !repo.knot_exists(knot_id).await? {
        return Err(ApiError::NotFound);
    }
    if input.bytes.is_empty() {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "file",
            "is required",
        )]));
    }
    let max_bytes = if spec.mime_type.starts_with("video/") {
        state.config().knots_media_storage.max_video_bytes
    } else {
        state.config().knots_media_storage.max_image_bytes
    };
    if input.bytes.len() as u64 > max_bytes {
        return Err(ApiError::PayloadTooLarge { max_bytes });
    }
    let declared_content_type = input
        .declared_content_type
        .as_deref()
        .unwrap_or(spec.mime_type)
        .trim()
        .to_ascii_lowercase();
    if declared_content_type != spec.mime_type {
        return Err(ApiError::UnsupportedMediaType(format!(
            "expected content type {}, got {}",
            spec.mime_type, declared_content_type
        )));
    }
    validate_magic(spec.mime_type, &input.bytes)?;

    let sha256 = sha256_hex(&input.bytes);
    let object_key = format!(
        "skills/knots/{knot_id}/{asset_id}/{sha256}.{}",
        spec.extension
    );
    let public_base_url = state
        .config()
        .knots_media_storage
        .public_base_url
        .trim_end_matches('/')
        .to_owned();
    let public_url = format!("{public_base_url}/{object_key}");
    let mut metadata = HashMap::from([
        ("sha256".to_owned(), sha256.clone()),
        ("source".to_owned(), "knots3d".to_owned()),
        ("knot_id".to_owned(), knot_id.to_owned()),
        ("asset_id".to_owned(), asset_id.to_owned()),
    ]);
    if let Some(filename) = input.original_filename.as_deref() {
        metadata.insert(
            "original_filename".to_owned(),
            safe_metadata_value(filename),
        );
    }
    if let Some(source_path) = input.source_path.as_deref() {
        metadata.insert("source_path".to_owned(), safe_metadata_value(source_path));
    }

    let put_result = state
        .object_store()
        .put_object(PutObjectRequest {
            bucket: Some(state.config().knots_media_storage.bucket.clone()),
            object_key: object_key.clone(),
            content_type: spec.mime_type.to_owned(),
            bytes: input.bytes,
            metadata,
            cache_control: Some(CACHE_CONTROL_IMMUTABLE.to_owned()),
        })
        .await
        .map_err(ApiError::internal)?;

    let media_resource_id =
        deterministic_media_id(&state.config().knots_media_storage.bucket, &object_key);
    let resource = MediaResourceDraft {
        id: media_resource_id.clone(),
        provider: PROVIDER_MINIO.to_owned(),
        storage_profile: state.config().knots_media_storage.storage_profile.clone(),
        bucket: state.config().knots_media_storage.bucket.clone(),
        object_key,
        public_base_url,
        public_url: public_url.clone(),
        mime_type: spec.mime_type.to_owned(),
        extension: spec.extension.to_owned(),
        size_bytes: i64::try_from(put_result.size_bytes).map_err(ApiError::internal)?,
        sha256_hex: sha256,
        etag: put_result.etag,
        width: None,
        height: None,
        duration_ms: None,
        status: "active".to_owned(),
        source_name: input.source_name,
        source_path: input.source_path,
        uploaded_by_user_id: Some(user.id.clone()),
    };
    let link = KnotMediaLinkDraft {
        knot_id: knot_id.to_owned(),
        asset_id: asset_id.to_owned(),
        media_type: spec.media_type.to_owned(),
        media_resource_id,
        sort_order: spec.sort_order,
        attribution: input.attribution,
        license_note: input.license_note,
    };
    let record = repo.upsert_knot_media(&resource, &link).await?;
    Ok(KnotMediaUploadResponse {
        status: "uploaded",
        knot_id: knot_id.to_owned(),
        media: KnotMediaAsset {
            id: asset_id.to_owned(),
            media_type: spec.media_type.to_owned(),
            url: record.public_url,
            mime_type: record.mime_type,
            width: record.width,
            height: record.height,
            size_bytes: record.size_bytes,
            attribution: link.attribution,
            license_note: link.license_note,
        },
    })
}

#[derive(Clone, Copy)]
struct AssetSpec {
    media_type: &'static str,
    mime_type: &'static str,
    extension: &'static str,
    sort_order: i32,
}

fn asset_spec(asset_id: &str) -> Option<AssetSpec> {
    match asset_id {
        "thumbnail" => Some(AssetSpec {
            media_type: "thumbnail",
            mime_type: "image/webp",
            extension: "webp",
            sort_order: 0,
        }),
        "preview" => Some(AssetSpec {
            media_type: "preview",
            mime_type: "image/webp",
            extension: "webp",
            sort_order: 1,
        }),
        "draw_gif" => Some(AssetSpec {
            media_type: "draw_gif",
            mime_type: "image/gif",
            extension: "gif",
            sort_order: 2,
        }),
        "turntable_gif" => Some(AssetSpec {
            media_type: "turntable_gif",
            mime_type: "image/gif",
            extension: "gif",
            sort_order: 3,
        }),
        "draw_mp4" => Some(AssetSpec {
            media_type: "draw_mp4",
            mime_type: "video/mp4",
            extension: "mp4",
            sort_order: 4,
        }),
        "turntable_mp4" => Some(AssetSpec {
            media_type: "turntable_mp4",
            mime_type: "video/mp4",
            extension: "mp4",
            sort_order: 5,
        }),
        _ => None,
    }
}

fn validate_safe_slug(field: &'static str, value: &str, max_len: usize) -> Result<(), ApiError> {
    let valid = !value.is_empty()
        && value.len() <= max_len
        && value.chars().enumerate().all(|(index, ch)| {
            ch.is_ascii_lowercase()
                || ch.is_ascii_digit()
                || (index > 0 && (ch == '-' || ch == '_'))
        });
    if valid {
        Ok(())
    } else {
        Err(ApiError::Validation(vec![FieldViolation::new(
            field,
            "must contain lowercase letters, numbers, hyphens, or underscores",
        )]))
    }
}

fn validate_magic(mime_type: &str, bytes: &[u8]) -> Result<(), ApiError> {
    let valid = match mime_type {
        "image/webp" => bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP",
        "image/gif" => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        "video/mp4" => bytes.windows(4).take(32).any(|window| window == b"ftyp"),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(ApiError::UnsupportedMediaType(format!(
            "file bytes do not match {mime_type}"
        )))
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn deterministic_media_id(bucket: &str, object_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bucket.as_bytes());
    hasher.update(b"/");
    hasher.update(object_key.as_bytes());
    format!("media-{}", &hex::encode(hasher.finalize())[..32])
}

fn safe_metadata_value(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_control())
        .take(256)
        .collect()
}
