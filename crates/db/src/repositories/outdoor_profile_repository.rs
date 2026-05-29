//! Persistence for account-level outdoor profile defaults.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, QueryResult};
use stellartrail_domain::outdoor_profile::OutdoorProfile;
use time::{OffsetDateTime, format_description::well_known::Iso8601};

use super::statement;

const OUTDOOR_PROFILE_SELECT: &str = "user_id, outdoor_id, real_name, gender, birth_date, height_cm, phone, \
    emergency_contact, emergency_contact_relationship, emergency_phone, blood_type, \
    medical_history, allergy_history, medical_response_note, diet_preference, \
    insurance_policy_no, insurance_company_phone, experience_note, created_at, updated_at";

/// Repository wrapper for authenticated users' reusable outdoor profile data.
#[derive(Clone)]
pub struct OutdoorProfileRepository {
    db: DatabaseConnection,
}

impl OutdoorProfileRepository {
    /// Creates a repository bound to the provided database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Returns the saved profile, or an empty profile when the user has not saved one.
    pub async fn get(&self, user_id: &str) -> Result<OutdoorProfile, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!(
                    "SELECT {OUTDOOR_PROFILE_SELECT} FROM user_outdoor_profiles WHERE user_id = ? LIMIT 1"
                ),
                vec![user_id.to_owned().into()],
            ))
            .await?;
        row.as_ref()
            .map(map_outdoor_profile)
            .transpose()
            .map(|profile| profile.unwrap_or_else(|| OutdoorProfile::empty(user_id)))
    }

    /// Upserts the normalized profile and returns the persisted row.
    pub async fn upsert(&self, profile: &OutdoorProfile) -> Result<OutdoorProfile, DbErr> {
        let now = now_rfc3339();
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "INSERT INTO user_outdoor_profiles (
                    user_id, outdoor_id, real_name, gender, birth_date, height_cm, phone,
                    emergency_contact, emergency_contact_relationship, emergency_phone, blood_type,
                    medical_history, allergy_history, medical_response_note, diet_preference,
                    insurance_policy_no, insurance_company_phone, experience_note, created_at,
                    updated_at
                 ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(user_id) DO UPDATE SET
                    outdoor_id = excluded.outdoor_id,
                    real_name = excluded.real_name,
                    gender = excluded.gender,
                    birth_date = excluded.birth_date,
                    height_cm = excluded.height_cm,
                    phone = excluded.phone,
                    emergency_contact = excluded.emergency_contact,
                    emergency_contact_relationship = excluded.emergency_contact_relationship,
                    emergency_phone = excluded.emergency_phone,
                    blood_type = excluded.blood_type,
                    medical_history = excluded.medical_history,
                    allergy_history = excluded.allergy_history,
                    medical_response_note = excluded.medical_response_note,
                    diet_preference = excluded.diet_preference,
                    insurance_policy_no = excluded.insurance_policy_no,
                    insurance_company_phone = excluded.insurance_company_phone,
                    experience_note = excluded.experience_note,
                    updated_at = excluded.updated_at",
                vec![
                    profile.user_id.clone().into(),
                    profile.outdoor_id.clone().into(),
                    profile.real_name.clone().into(),
                    profile.gender.clone().into(),
                    profile.birth_date.clone().into(),
                    profile.height_cm.into(),
                    profile.phone.clone().into(),
                    profile.emergency_contact.clone().into(),
                    profile.emergency_contact_relationship.clone().into(),
                    profile.emergency_phone.clone().into(),
                    profile.blood_type.clone().into(),
                    profile.medical_history.clone().into(),
                    profile.allergy_history.clone().into(),
                    profile.medical_response_note.clone().into(),
                    profile.diet_preference.clone().into(),
                    profile.insurance_policy_no.clone().into(),
                    profile.insurance_company_phone.clone().into(),
                    profile.experience_note.clone().into(),
                    now.clone().into(),
                    now.into(),
                ],
            ))
            .await?;
        self.get(&profile.user_id).await
    }
}

fn map_outdoor_profile(row: &QueryResult) -> Result<OutdoorProfile, DbErr> {
    Ok(OutdoorProfile {
        user_id: row.try_get("", "user_id")?,
        outdoor_id: row.try_get("", "outdoor_id")?,
        real_name: row.try_get("", "real_name")?,
        gender: row.try_get("", "gender")?,
        birth_date: row.try_get("", "birth_date")?,
        height_cm: row.try_get("", "height_cm")?,
        phone: row.try_get("", "phone")?,
        emergency_contact: row.try_get("", "emergency_contact")?,
        emergency_contact_relationship: row.try_get("", "emergency_contact_relationship")?,
        emergency_phone: row.try_get("", "emergency_phone")?,
        blood_type: row.try_get("", "blood_type")?,
        medical_history: row.try_get("", "medical_history")?,
        allergy_history: row.try_get("", "allergy_history")?,
        medical_response_note: row.try_get("", "medical_response_note")?,
        diet_preference: row.try_get("", "diet_preference")?,
        insurance_policy_no: row.try_get("", "insurance_policy_no")?,
        insurance_company_phone: row.try_get("", "insurance_company_phone")?,
        experience_note: row.try_get("", "experience_note")?,
        created_at: Some(row.try_get("", "created_at")?),
        updated_at: Some(row.try_get("", "updated_at")?),
    })
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Iso8601::DEFAULT)
        .expect("format current timestamp")
}
