//! HTTP DTOs for administrator-managed knot media uploads.

use serde::{Deserialize, Serialize};
use stellartrail_domain::skill::KnotMediaAsset;

/// Response returned after a knot media asset has been written to object storage and linked in DB.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KnotMediaUploadResponse {
    pub status: &'static str,
    pub knot_id: String,
    pub media: KnotMediaAsset,
}
