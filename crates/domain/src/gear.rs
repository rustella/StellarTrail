use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearItem {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub category: String,
    pub weight_g: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouteGearSuggestion {
    pub route_id: String,
    pub gear_category: String,
    pub gear_name: String,
    pub required_level: RequiredLevel,
    pub reason: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RequiredLevel {
    Required,
    Recommended,
    Optional,
}
