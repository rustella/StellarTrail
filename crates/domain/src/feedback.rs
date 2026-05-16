//! 用户反馈领域模块，负责反馈分类和文本字段规范化。

use serde::{Deserialize, Serialize};

use crate::validation::{
    FieldViolation, ValidationError, normalize_optional_text, normalize_required_text,
};

/// 用户反馈分类。
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackCategory {
    Bug,
    Suggestion,
    ContentCorrection,
    Other,
}

impl FeedbackCategory {
    /// 将 API 字符串转换为反馈分类。
    pub fn from_key(value: &str) -> Option<Self> {
        match value {
            "bug" => Some(Self::Bug),
            "suggestion" => Some(Self::Suggestion),
            "content_correction" => Some(Self::ContentCorrection),
            "other" => Some(Self::Other),
            _ => None,
        }
    }

    /// 返回 API/DB 存储使用的稳定 key。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bug => "bug",
            Self::Suggestion => "suggestion",
            Self::ContentCorrection => "content_correction",
            Self::Other => "other",
        }
    }
}

/// 经校验的反馈草稿。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeedbackDraft {
    pub category: FeedbackCategory,
    pub content: String,
    pub contact: Option<String>,
    pub page: Option<String>,
    pub client_platform: Option<String>,
    pub client_version: Option<String>,
    pub device_model: Option<String>,
}

/// 校验并规范化反馈字段。
pub fn validate_feedback_draft(
    category: String,
    content: String,
    contact: Option<String>,
    page: Option<String>,
    client_platform: Option<String>,
    client_version: Option<String>,
    device_model: Option<String>,
) -> Result<FeedbackDraft, ValidationError> {
    let mut errors = Vec::new();
    let category = FeedbackCategory::from_key(category.trim()).unwrap_or_else(|| {
        errors.push(FieldViolation::new("category", "is not supported"));
        FeedbackCategory::Other
    });
    let content = normalize_required_text(content, 2000, "content", &mut errors);
    let contact = normalize_optional_text(contact, 120, "contact", &mut errors);
    let page = normalize_optional_text(page, 200, "page", &mut errors);
    let client_platform =
        normalize_optional_text(client_platform, 64, "client_platform", &mut errors);
    let client_version = normalize_optional_text(client_version, 64, "client_version", &mut errors);
    let device_model = normalize_optional_text(device_model, 120, "device_model", &mut errors);

    if errors.is_empty() {
        Ok(FeedbackDraft {
            category,
            content,
            contact,
            page,
            client_platform,
            client_version,
            device_model,
        })
    } else {
        Err(ValidationError::new(errors))
    }
}
