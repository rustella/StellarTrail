//! 装备库业务服务模块，在 repository 读写之外补充领域校验、缓存失效和导出列表编排。

use stellartrail_db::repositories::{GearRepository, ListGearOptions};
use stellartrail_domain::gear::{GearDraft, GearItem, GearTab};

use crate::{dto::gear::UpdateGearRequest, error::ApiError, state::AppState};

/// 执行 `create gear` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
pub async fn create_gear(
    state: &AppState,
    user_id: &str,
    mut draft: GearDraft,
) -> Result<GearItem, ApiError> {
    draft.validate_and_normalize()?;
    GearRepository::new(state.db().clone())
        .create(user_id, &draft)
        .await
        .map_err(ApiError::from)
}

/// 执行 `update gear` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
pub async fn update_gear(
    state: &AppState,
    user_id: &str,
    id: &str,
    request: UpdateGearRequest,
) -> Result<GearItem, ApiError> {
    let repo = GearRepository::new(state.db().clone());
    let existing = repo.get(user_id, id).await?.ok_or(ApiError::NotFound)?;
    let mut draft = request.merge_into(&existing);
    draft.validate_and_normalize()?;
    repo.replace(user_id, id, &draft)
        .await?
        .ok_or(ApiError::NotFound)
}

/// 执行 `list for export` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
pub async fn list_for_export(
    state: &AppState,
    user_id: &str,
    tab: GearTab,
) -> Result<Vec<GearItem>, ApiError> {
    let repo = GearRepository::new(state.db().clone());
    let (items, _) = repo
        .list(
            user_id,
            &ListGearOptions {
                tab,
                limit: 10_000,
                ..Default::default()
            },
        )
        .await?;
    Ok(items)
}
