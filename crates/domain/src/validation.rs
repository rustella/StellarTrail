//! 领域校验辅助模块，提供字段级错误和文本规范化工具。

use serde::Serialize;
use thiserror::Error;

/// FieldViolation 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FieldViolation {
    pub field: String,
    pub message: String,
}

impl FieldViolation {
    /// 执行 `new` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// ValidationError 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Clone, Debug, Error)]
#[error("request validation failed")]
pub struct ValidationError {
    pub fields: Vec<FieldViolation>,
}

impl ValidationError {
    /// 执行 `new` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn new(fields: Vec<FieldViolation>) -> Self {
        Self { fields }
    }

    /// 执行 `single` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn single(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(vec![FieldViolation::new(field, message)])
    }

    /// 执行 `is empty` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// 执行 `normalize optional text` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
pub fn normalize_optional_text(
    value: Option<String>,
    max_chars: usize,
    field: &str,
    errors: &mut Vec<FieldViolation>,
) -> Option<String> {
    let raw = value?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.chars().count() > max_chars {
        errors.push(FieldViolation::new(
            field,
            format!("must be at most {max_chars} characters"),
        ));
    }
    Some(trimmed.to_owned())
}

/// 执行 `normalize required text` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
pub fn normalize_required_text(
    value: String,
    max_chars: usize,
    field: &str,
    errors: &mut Vec<FieldViolation>,
) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        errors.push(FieldViolation::new(field, "is required"));
    }
    if trimmed.chars().count() > max_chars {
        errors.push(FieldViolation::new(
            field,
            format!("must be at most {max_chars} characters"),
        ));
    }
    trimmed.to_owned()
}
