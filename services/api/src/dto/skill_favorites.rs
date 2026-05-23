//! HTTP DTOs for authenticated skill favorite routes.

use serde::Serialize;
use stellartrail_domain::skill::{KnotSummary, Locale, PageInfo};

/// Filter option returned for the favorite skills list UI.
#[derive(Clone, Debug, Serialize)]
pub struct FavoriteSkillFilterResponse {
    pub id: String,
    pub title: String,
    pub count: u32,
}

/// One favorite knot item returned by the favorite skills list.
#[derive(Clone, Debug, Serialize)]
pub struct FavoriteKnotItemResponse {
    pub skill_category: &'static str,
    pub favorited_at: String,
    pub knot: KnotSummary,
}

/// Paginated current-user favorite skills response.
#[derive(Clone, Debug, Serialize)]
pub struct ListFavoriteSkillsResponse {
    pub locale: Locale,
    pub filters: Vec<FavoriteSkillFilterResponse>,
    pub items: Vec<FavoriteKnotItemResponse>,
    pub page: PageInfo,
}

/// Current-user favorite status for one knot.
#[derive(Clone, Debug, Serialize)]
pub struct FavoriteKnotStatusResponse {
    pub skill_category: &'static str,
    pub knot_id: String,
    pub is_favorited: bool,
    pub favorited_at: Option<String>,
}
