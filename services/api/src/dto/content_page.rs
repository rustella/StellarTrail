//! Content page HTTP DTOs for DB-backed client copy.

use serde::{Deserialize, Serialize};
use stellartrail_db::repositories::AppContentPageRecord;

/// Query parameters accepted by the public content page endpoint.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContentPageQuery {
    pub client_key: Option<String>,
    pub locale: Option<String>,
}

/// One structured copy section in a content page.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContentPageSection {
    pub icon: String,
    pub title: String,
    pub body: String,
}

/// JSON document stored in `app_content_pages.content_json`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContentPageContent {
    pub eyebrow: String,
    pub title: String,
    pub subtitle: String,
    pub sections: Vec<ContentPageSection>,
    pub button_text: String,
}

/// Public response for one DB-backed content page.
#[derive(Clone, Debug, Serialize)]
pub struct ContentPageResponse {
    pub page_key: String,
    pub client_key: String,
    pub locale: String,
    pub eyebrow: String,
    pub title: String,
    pub subtitle: String,
    pub sections: Vec<ContentPageSection>,
    pub button_text: String,
    pub updated_at: String,
}

impl ContentPageResponse {
    /// Converts a persisted row and parsed JSON document into the public response shape.
    pub fn from_record(record: &AppContentPageRecord, content: ContentPageContent) -> Self {
        Self {
            page_key: record.page_key.clone(),
            client_key: record.client_key.clone(),
            locale: record.locale.clone(),
            eyebrow: content.eyebrow,
            title: content.title,
            subtitle: content.subtitle,
            sections: content.sections,
            button_text: content.button_text,
            updated_at: record.updated_at.clone(),
        }
    }
}
