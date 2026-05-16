//! 用户领域模型，描述当前用户资料在 API 与数据库之间流转的核心字段。

use serde::{Deserialize, Serialize};

/// UserProfile 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserProfile {
    pub id: String,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
}
