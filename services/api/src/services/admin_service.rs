//! Database-backed administrator authorization and role management service.
//!
//! All administrator decisions resolve the current bearer-token user to an
//! `admin_roles` row. Configuration allowlists are intentionally not consulted,
//! which keeps production permission changes auditable in the database.

use stellartrail_db::repositories::{AdminRoleRecord, AdminRoleRepository, UserRecord};
use stellartrail_domain::{admin::AdminRole, validation::FieldViolation};

use crate::{error::ApiError, state::AppState};

/// Selector accepted by administrator-management APIs after HTTP decoding.
#[derive(Clone, Debug, Default)]
pub struct AdminUserSelectorInput {
    pub username: Option<String>,
    pub user_id: Option<String>,
}

/// Result returned after a super administrator grants a role.
#[derive(Clone, Debug)]
pub struct GrantAdminServiceResult {
    pub record: AdminRoleRecord,
    pub created: bool,
}

/// Ensures the current user has either `admin` or `super_admin`.
pub async fn ensure_admin(
    state: &AppState,
    user: &UserRecord,
) -> Result<AdminRoleRecord, ApiError> {
    let record = AdminRoleRepository::new(state.db().clone())
        .find_role(&user.id)
        .await?
        .ok_or(ApiError::Forbidden)?;
    if record.role.can_administer() {
        Ok(record)
    } else {
        Err(ApiError::Forbidden)
    }
}

/// Ensures the current user is a `super_admin`.
pub async fn ensure_super_admin(
    state: &AppState,
    user: &UserRecord,
) -> Result<AdminRoleRecord, ApiError> {
    let record = ensure_admin(state, user).await?;
    if record.role.can_manage_admins() {
        Ok(record)
    } else {
        Err(ApiError::Forbidden)
    }
}

/// Grants regular `admin` to an existing non-deleted user.
pub async fn grant_admin(
    state: &AppState,
    actor: &UserRecord,
    selector: AdminUserSelectorInput,
) -> Result<GrantAdminServiceResult, ApiError> {
    ensure_super_admin(state, actor).await?;
    let repo = AdminRoleRepository::new(state.db().clone());
    let target = resolve_target_user(&repo, selector).await?;
    let result = repo.grant_admin(&target.id, &actor.id).await?;
    Ok(GrantAdminServiceResult {
        record: result.record,
        created: result.created,
    })
}

/// Revokes a regular `admin` role from an existing non-deleted user.
pub async fn revoke_admin(
    state: &AppState,
    actor: &UserRecord,
    selector: AdminUserSelectorInput,
) -> Result<(), ApiError> {
    ensure_super_admin(state, actor).await?;
    let repo = AdminRoleRepository::new(state.db().clone());
    let target = resolve_target_user(&repo, selector).await?;
    let Some(existing) = repo.find_role(&target.id).await? else {
        return Err(ApiError::NotFound);
    };
    if existing.role == AdminRole::SuperAdmin {
        return Err(ApiError::Validation(vec![FieldViolation::new(
            "role",
            "super_admin cannot be revoked by this endpoint",
        )]));
    }
    if repo.revoke_admin(&target.id).await? {
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}

async fn resolve_target_user(
    repo: &AdminRoleRepository,
    selector: AdminUserSelectorInput,
) -> Result<stellartrail_db::repositories::AdminTargetUser, ApiError> {
    let selector = normalize_selector(selector)?;
    match selector {
        NormalizedAdminUserSelector::UserId(user_id) => repo
            .find_target_user_by_id(&user_id)
            .await?
            .ok_or(ApiError::NotFound),
        NormalizedAdminUserSelector::Username(username) => repo
            .find_target_user_by_username(&username)
            .await?
            .ok_or(ApiError::NotFound),
    }
}

enum NormalizedAdminUserSelector {
    UserId(String),
    Username(String),
}

fn normalize_selector(
    selector: AdminUserSelectorInput,
) -> Result<NormalizedAdminUserSelector, ApiError> {
    match (selector.username, selector.user_id) {
        (Some(username), None) => Ok(NormalizedAdminUserSelector::Username(normalize_username(
            username,
        )?)),
        (None, Some(user_id)) => Ok(NormalizedAdminUserSelector::UserId(normalize_user_id(
            user_id,
        )?)),
        (None, None) => Err(ApiError::Validation(vec![FieldViolation::new(
            "selector",
            "username or user_id is required",
        )])),
        (Some(_), Some(_)) => Err(ApiError::Validation(vec![FieldViolation::new(
            "selector",
            "provide exactly one of username or user_id",
        )])),
    }
}

fn normalize_user_id(user_id: String) -> Result<String, ApiError> {
    let user_id = user_id.trim().to_owned();
    if user_id.is_empty() {
        Err(ApiError::Validation(vec![FieldViolation::new(
            "user_id",
            "is required",
        )]))
    } else {
        Ok(user_id)
    }
}

fn normalize_username(username: String) -> Result<String, ApiError> {
    let username = username.trim().to_ascii_lowercase();
    let mut errors = Vec::new();
    let len = username.chars().count();
    if username.is_empty() {
        errors.push(FieldViolation::new("username", "is required"));
    } else {
        if !(3..=32).contains(&len) {
            errors.push(FieldViolation::new(
                "username",
                "must be between 3 and 32 characters",
            ));
        }
        if !username
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        {
            errors.push(FieldViolation::new(
                "username",
                "only letters, numbers, underscores and hyphens are allowed",
            ));
        }
    }
    if errors.is_empty() {
        Ok(username)
    } else {
        Err(ApiError::Validation(errors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_selector_requires_exactly_one_identifier() {
        assert!(normalize_selector(AdminUserSelectorInput::default()).is_err());
        assert!(
            normalize_selector(AdminUserSelectorInput {
                username: Some("trail_admin".to_owned()),
                user_id: Some("user-id".to_owned()),
            })
            .is_err()
        );
    }

    #[test]
    fn normalize_selector_lowercases_usernames() {
        let selector = normalize_selector(AdminUserSelectorInput {
            username: Some(" Trail_Admin ".to_owned()),
            user_id: None,
        })
        .unwrap();
        match selector {
            NormalizedAdminUserSelector::Username(username) => assert_eq!(username, "trail_admin"),
            NormalizedAdminUserSelector::UserId(_) => panic!("expected username selector"),
        }
    }
}
