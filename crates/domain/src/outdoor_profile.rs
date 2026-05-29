//! Account-level outdoor profile defaults used by team trip plans.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use time::{Date, OffsetDateTime, macros::format_description};

use crate::validation::{
    FieldViolation, ValidationError, normalize_optional_text, normalize_required_text,
};

const BIRTH_DATE_FORMAT: &[time::format_description::FormatItem<'_>] =
    format_description!("[year]-[month]-[day]");

/// Account-level outdoor profile that can be copied into a trip member.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OutdoorProfile {
    pub user_id: String,
    pub outdoor_id: Option<String>,
    pub real_name: Option<String>,
    pub gender: Option<String>,
    pub birth_date: Option<String>,
    pub height_cm: Option<i32>,
    pub phone: Option<String>,
    pub emergency_contact: Option<String>,
    pub emergency_contact_relationship: Option<String>,
    pub emergency_phone: Option<String>,
    pub blood_type: Option<String>,
    pub medical_history: Option<String>,
    pub allergy_history: Option<String>,
    pub medical_response_note: Option<String>,
    pub diet_preference: Option<String>,
    pub insurance_policy_no: Option<String>,
    pub insurance_company_phone: Option<String>,
    pub experience_note: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl OutdoorProfile {
    /// Builds an empty profile response for users who have not saved defaults yet.
    pub fn empty(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            ..Self::default()
        }
    }

    /// Applies sparse API changes where missing fields keep their current values and null clears.
    pub fn apply_sparse_patch(
        &mut self,
        changes: BTreeMap<String, JsonValue>,
    ) -> Result<(), ValidationError> {
        let mut errors = Vec::new();
        for (field, value) in changes {
            match field.as_str() {
                "outdoor_id" => {
                    assign_optional_text(&mut self.outdoor_id, &field, value, &mut errors)
                }
                "real_name" => {
                    assign_optional_text(&mut self.real_name, &field, value, &mut errors)
                }
                "gender" => assign_optional_text(&mut self.gender, &field, value, &mut errors),
                "birth_date" => {
                    assign_optional_text(&mut self.birth_date, &field, value, &mut errors);
                }
                "height_cm" => {
                    assign_optional_i32(&mut self.height_cm, &field, value, &mut errors);
                }
                "phone" => assign_optional_text(&mut self.phone, &field, value, &mut errors),
                "emergency_contact" => {
                    assign_optional_text(&mut self.emergency_contact, &field, value, &mut errors);
                }
                "emergency_contact_relationship" => {
                    assign_optional_text(
                        &mut self.emergency_contact_relationship,
                        &field,
                        value,
                        &mut errors,
                    );
                }
                "emergency_phone" => {
                    assign_optional_text(&mut self.emergency_phone, &field, value, &mut errors);
                }
                "blood_type" => {
                    assign_optional_text(&mut self.blood_type, &field, value, &mut errors);
                }
                "medical_history" => {
                    assign_optional_text(&mut self.medical_history, &field, value, &mut errors);
                }
                "allergy_history" => {
                    assign_optional_text(&mut self.allergy_history, &field, value, &mut errors);
                }
                "medical_response_note" => {
                    assign_optional_text(
                        &mut self.medical_response_note,
                        &field,
                        value,
                        &mut errors,
                    );
                }
                "diet_preference" => {
                    assign_optional_text(&mut self.diet_preference, &field, value, &mut errors);
                }
                "insurance_policy_no" => {
                    assign_optional_text(&mut self.insurance_policy_no, &field, value, &mut errors);
                }
                "insurance_company_phone" => {
                    assign_optional_text(
                        &mut self.insurance_company_phone,
                        &field,
                        value,
                        &mut errors,
                    );
                }
                "experience_note" => {
                    assign_optional_text(&mut self.experience_note, &field, value, &mut errors);
                }
                _ => errors.push(FieldViolation::new(field, "is not editable")),
            }
        }
        if !errors.is_empty() {
            return Err(ValidationError::new(errors));
        }
        self.validate_and_normalize()
    }

    /// Normalizes user-entered outdoor defaults and enforces conservative limits.
    pub fn validate_and_normalize(&mut self) -> Result<(), ValidationError> {
        let mut errors = Vec::new();
        normalize_common_profile_fields(
            &mut self.outdoor_id,
            &mut self.real_name,
            &mut self.gender,
            &mut self.birth_date,
            &mut self.height_cm,
            &mut self.phone,
            &mut self.emergency_contact,
            &mut self.emergency_contact_relationship,
            &mut self.emergency_phone,
            &mut self.blood_type,
            &mut self.medical_history,
            &mut self.allergy_history,
            &mut self.medical_response_note,
            &mut self.diet_preference,
            &mut self.insurance_policy_no,
            &mut self.insurance_company_phone,
            &mut self.experience_note,
            &mut errors,
        );
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::new(errors))
        }
    }
}

/// Shared member/profile fields copied between account defaults and plan members.
#[allow(clippy::too_many_arguments)]
pub fn normalize_common_profile_fields(
    outdoor_id: &mut Option<String>,
    real_name: &mut Option<String>,
    gender: &mut Option<String>,
    birth_date: &mut Option<String>,
    height_cm: &mut Option<i32>,
    phone: &mut Option<String>,
    emergency_contact: &mut Option<String>,
    emergency_contact_relationship: &mut Option<String>,
    emergency_phone: &mut Option<String>,
    blood_type: &mut Option<String>,
    medical_history: &mut Option<String>,
    allergy_history: &mut Option<String>,
    medical_response_note: &mut Option<String>,
    diet_preference: &mut Option<String>,
    insurance_policy_no: &mut Option<String>,
    insurance_company_phone: &mut Option<String>,
    experience_note: &mut Option<String>,
    errors: &mut Vec<FieldViolation>,
) {
    *outdoor_id = normalize_optional_text(outdoor_id.take(), 80, "outdoor_id", errors);
    *real_name = normalize_optional_text(real_name.take(), 80, "real_name", errors);
    *gender = normalize_optional_text(gender.take(), 20, "gender", errors);
    *birth_date = normalize_optional_birth_date(birth_date.take(), errors);
    *phone = normalize_optional_text(phone.take(), 40, "phone", errors);
    *emergency_contact =
        normalize_optional_text(emergency_contact.take(), 80, "emergency_contact", errors);
    *emergency_contact_relationship = normalize_optional_text(
        emergency_contact_relationship.take(),
        40,
        "emergency_contact_relationship",
        errors,
    );
    *emergency_phone =
        normalize_optional_text(emergency_phone.take(), 40, "emergency_phone", errors);
    *blood_type = normalize_optional_text(blood_type.take(), 20, "blood_type", errors);
    *medical_history =
        normalize_optional_text(medical_history.take(), 500, "medical_history", errors);
    *allergy_history =
        normalize_optional_text(allergy_history.take(), 500, "allergy_history", errors);
    *medical_response_note = normalize_optional_text(
        medical_response_note.take(),
        1000,
        "medical_response_note",
        errors,
    );
    *diet_preference =
        normalize_optional_text(diet_preference.take(), 500, "diet_preference", errors);
    *insurance_policy_no = normalize_optional_text(
        insurance_policy_no.take(),
        120,
        "insurance_policy_no",
        errors,
    );
    *insurance_company_phone = normalize_optional_text(
        insurance_company_phone.take(),
        40,
        "insurance_company_phone",
        errors,
    );
    *experience_note =
        normalize_optional_text(experience_note.take(), 1000, "experience_note", errors);
    if let Some(value) = height_cm
        && !matches!(*value, 50..=250)
    {
        errors.push(FieldViolation::new(
            "height_cm",
            "must be between 50 and 250",
        ));
    }
}

fn normalize_optional_birth_date(
    value: Option<String>,
    errors: &mut Vec<FieldViolation>,
) -> Option<String> {
    let normalized = normalize_optional_text(value, 10, "birth_date", errors);
    let Some(date_text) = normalized.as_deref() else {
        return normalized;
    };
    let parsed = match Date::parse(date_text, BIRTH_DATE_FORMAT) {
        Ok(date) => date,
        Err(_) => {
            errors.push(FieldViolation::new(
                "birth_date",
                "must be a valid YYYY-MM-DD date",
            ));
            return normalized;
        }
    };
    let min_date = time::macros::date!(1900 - 01 - 01);
    let today = OffsetDateTime::now_utc().date();
    if parsed < min_date {
        errors.push(FieldViolation::new(
            "birth_date",
            "must be on or after 1900-01-01",
        ));
    }
    if parsed > today {
        errors.push(FieldViolation::new("birth_date", "cannot be in the future"));
    }
    normalized
}

/// Normalizes and validates a required display name for a plan member profile.
pub fn normalize_display_name(value: String, errors: &mut Vec<FieldViolation>) -> String {
    normalize_required_text(value, 80, "display_name", errors)
}

fn assign_optional_text(
    target: &mut Option<String>,
    field: &str,
    value: JsonValue,
    errors: &mut Vec<FieldViolation>,
) {
    match value {
        JsonValue::Null => *target = None,
        JsonValue::String(value) => *target = Some(value),
        _ => errors.push(FieldViolation::new(field, "must be a string or null")),
    }
}

fn assign_optional_i32(
    target: &mut Option<i32>,
    field: &str,
    value: JsonValue,
    errors: &mut Vec<FieldViolation>,
) {
    match value {
        JsonValue::Null => *target = None,
        JsonValue::Number(value) => {
            match value.as_i64().and_then(|value| i32::try_from(value).ok()) {
                Some(value) => *target = Some(value),
                None => errors.push(FieldViolation::new(field, "must be an integer")),
            }
        }
        _ => errors.push(FieldViolation::new(field, "must be an integer or null")),
    }
}
