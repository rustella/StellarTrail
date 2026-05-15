use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserProfile {
    pub id: String,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
}
