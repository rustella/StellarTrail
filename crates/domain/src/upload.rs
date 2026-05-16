//! 上传图片校验领域模块，统一限制格式、扩展名、声明 MIME 与 magic/mask 签名。

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::validation::ValidationError;

/// 服务端允许的图片类型白名单。
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageType {
    Jpeg,
    Png,
    Webp,
}

impl ImageType {
    /// 返回服务端检测后可信的 Content-Type。
    pub fn content_type(self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Webp => "image/webp",
        }
    }

    /// 返回服务端生成 object key 时使用的安全扩展名。
    pub fn safe_extension(self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Webp => "webp",
        }
    }

    /// 返回 API 响应中稳定的图片类型字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Jpeg => "jpeg",
            Self::Png => "png",
            Self::Webp => "webp",
        }
    }
}

/// 通过服务端校验后的上传图片元数据。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedImageUpload {
    pub image_type: ImageType,
    pub original_filename: String,
    pub content_type: String,
    pub safe_extension: &'static str,
}

/// 校验上传图片的文件名后缀、声明 Content-Type 与文件 magic/mask 是否一致。
pub fn validate_image_upload(
    original_filename: Option<&str>,
    declared_content_type: Option<&str>,
    bytes: &[u8],
) -> Result<ValidatedImageUpload, ValidationError> {
    if bytes.is_empty() {
        return Err(ValidationError::single("file", "must not be empty"));
    }

    let original_filename = sanitize_filename(original_filename)?;
    let extension_type = extension_to_image_type(&original_filename)
        .ok_or_else(|| ValidationError::single("file", "unsupported image extension"))?;
    let detected_type = detect_image_type(bytes)
        .ok_or_else(|| ValidationError::single("file", "unsupported or invalid image content"))?;

    if extension_type != detected_type {
        return Err(ValidationError::single(
            "file",
            "file extension does not match image signature",
        ));
    }

    if let Some(declared_type) = declared_content_type.and_then(content_type_to_image_type) {
        if declared_type != detected_type {
            return Err(ValidationError::single(
                "file",
                "declared content type does not match image signature",
            ));
        }
    } else if let Some(raw_content_type) = declared_content_type {
        if !raw_content_type.trim().is_empty() {
            return Err(ValidationError::single(
                "file",
                "unsupported declared content type",
            ));
        }
    }

    Ok(ValidatedImageUpload {
        image_type: detected_type,
        original_filename,
        content_type: detected_type.content_type().to_owned(),
        safe_extension: detected_type.safe_extension(),
    })
}

fn sanitize_filename(original_filename: Option<&str>) -> Result<String, ValidationError> {
    let filename = original_filename
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ValidationError::single("file", "filename is required"))?;
    if filename.chars().count() > 160 {
        return Err(ValidationError::single(
            "file",
            "filename must be at most 160 characters",
        ));
    }
    if filename.contains('/') || filename.contains('\\') || filename.contains('\0') {
        return Err(ValidationError::single(
            "file",
            "filename must not contain path separators",
        ));
    }
    if filename.chars().any(|ch| ch.is_control()) {
        return Err(ValidationError::single(
            "file",
            "filename must not contain control characters",
        ));
    }
    Ok(filename.to_owned())
}

fn extension_to_image_type(filename: &str) -> Option<ImageType> {
    let extension = Path::new(filename)
        .extension()?
        .to_str()?
        .to_ascii_lowercase();
    match extension.as_str() {
        "jpg" | "jpeg" => Some(ImageType::Jpeg),
        "png" => Some(ImageType::Png),
        "webp" => Some(ImageType::Webp),
        _ => None,
    }
}

fn content_type_to_image_type(value: &str) -> Option<ImageType> {
    match value.trim().to_ascii_lowercase().as_str() {
        "image/jpeg" | "image/jpg" => Some(ImageType::Jpeg),
        "image/png" => Some(ImageType::Png),
        "image/webp" => Some(ImageType::Webp),
        _ => None,
    }
}

fn detect_image_type(bytes: &[u8]) -> Option<ImageType> {
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Some(ImageType::Jpeg);
    }
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]) {
        return Some(ImageType::Png);
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some(ImageType::Webp);
    }
    None
}
