//! Roadmap HTTP DTOs for public planning content, administrator maintenance, and user interest state.

use serde::{Deserialize, Serialize};
use stellartrail_db::repositories::RoadmapListEntry;
use stellartrail_domain::roadmap::RoadmapItemDraft;

use crate::services::roadmap_service::RoadmapListInput;

/// Public, current-user, and administrator list query.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RoadmapListQuery {
    pub client_key: Option<String>,
    pub status: Option<String>,
    pub limit: Option<u64>,
    pub cursor: Option<String>,
}

impl From<RoadmapListQuery> for RoadmapListInput {
    fn from(value: RoadmapListQuery) -> Self {
        Self {
            client_key: value.client_key,
            status: value.status,
            limit: value.limit,
            cursor: value.cursor,
        }
    }
}

/// Administrator create/update request body.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoadmapItemRequest {
    pub client_key: String,
    pub title: String,
    pub summary: String,
    pub details: Option<String>,
    pub category: String,
    pub status: String,
    pub priority: i32,
    pub sort_order: i32,
    pub is_published: bool,
}

impl From<RoadmapItemRequest> for RoadmapItemDraft {
    fn from(value: RoadmapItemRequest) -> Self {
        Self {
            client_key: value.client_key,
            title: value.title,
            summary: value.summary,
            details: value.details,
            category: value.category,
            status: value.status,
            priority: value.priority,
            sort_order: value.sort_order,
            is_published: value.is_published,
        }
    }
}

/// Roadmap item response with aggregate counts and current-user state.
#[derive(Clone, Debug, Serialize)]
pub struct RoadmapItemResponse {
    pub id: String,
    pub client_key: String,
    pub title: String,
    pub summary: String,
    pub details: Option<String>,
    pub category: String,
    pub status: String,
    pub priority: i32,
    pub sort_order: i32,
    pub is_published: bool,
    pub vote_count: u32,
    pub subscription_count: u32,
    pub is_voted: bool,
    pub is_subscribed: bool,
    pub published_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl RoadmapItemResponse {
    /// Converts a repository entry to the stable API response shape.
    pub fn from_entry(entry: &RoadmapListEntry) -> Self {
        let item = &entry.item;
        Self {
            id: item.id.clone(),
            client_key: item.client_key.clone(),
            title: item.title.clone(),
            summary: item.summary.clone(),
            details: item.details.clone(),
            category: item.category.clone(),
            status: item.status.clone(),
            priority: item.priority,
            sort_order: item.sort_order,
            is_published: item.is_published,
            vote_count: entry.vote_count,
            subscription_count: entry.subscription_count,
            is_voted: entry.is_voted,
            is_subscribed: entry.is_subscribed,
            published_at: item.published_at.clone(),
            created_at: item.created_at.clone(),
            updated_at: item.updated_at.clone(),
        }
    }
}

/// Paginated Roadmap list response.
#[derive(Clone, Debug, Serialize)]
pub struct ListRoadmapResponse {
    pub items: Vec<RoadmapItemResponse>,
    pub next_cursor: Option<String>,
}

impl ListRoadmapResponse {
    /// Converts a page of repository entries to a response.
    pub fn from_entries(entries: &[RoadmapListEntry], next_cursor: Option<String>) -> Self {
        Self {
            items: entries
                .iter()
                .map(RoadmapItemResponse::from_entry)
                .collect(),
            next_cursor,
        }
    }
}

/// Current-user vote or subscription response.
pub type RoadmapInteractionStatusResponse = RoadmapItemResponse;
