//! Domain validation helper module providing field-level errors and text normalization utilities.

use serde::Serialize;
use thiserror::Error;

/// Stable data boundary for `FieldViolation`, exposed by or reused within this module.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FieldViolation {
    pub field: String,
    pub message: String,
}

impl FieldViolation {
    /// Runs the `new` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Stable data boundary for `ValidationError`, exposed by or reused within this module.
#[derive(Clone, Debug, Error)]
#[error("request validation failed")]
pub struct ValidationError {
    pub fields: Vec<FieldViolation>,
}

impl ValidationError {
    /// Runs the `new` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn new(fields: Vec<FieldViolation>) -> Self {
        Self { fields }
    }

    /// Runs the `single` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn single(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(vec![FieldViolation::new(field, message)])
    }

    /// Runs the `is empty` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// Runs the `normalize optional text` server-side flow while preserving input validation, error propagation, and state invariants.
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

/// Runs the `normalize required text` server-side flow while preserving input validation, error propagation, and state invariants.
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
