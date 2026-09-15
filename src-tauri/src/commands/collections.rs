// 合集命令
use tauri::State;

use crate::auth::{require_feature, FEATURE_COLLECTION, FEATURE_COLLECTION_OWN};
use crate::db::repository;
use crate::error::{AppError, CmdResult};
use crate::state::AppState;

pub use repository::collections::{Collection, CreateCollection};

/// 选择合集写操作的权限：作者本人用「合集(自己)」，他人用「合集(管理)」
async fn require_collection_write(
    state: &AppState,
    collection_user_id: Option<i64>,
) -> Result<crate::auth::UserSession, AppError> {
    let user = crate::auth::require_login(&state.auth).await?;
    let is_own = collection_user_id.map(|uid| uid == user.id).unwrap_or(false);
    let key = if is_own {
        FEATURE_COLLECTION_OWN
    } else {
        FEATURE_COLLECTION
    };
    require_feature(&state.db, &state.auth, key).await
}

#[tauri::command]
pub async fn list_collections(state: State<'_, AppState>) -> CmdResult<Vec<Collection>> {
    Ok(repository::collections::list(&state.db).await?)
}

#[tauri::command]
pub async fn list_collections_page(
    state: State<'_, AppState>,
    page: i64,
    page_size: i64,
    search: Option<String>,
) -> CmdResult<repository::Paged<Collection>> {
    let (page, page_size) = repository::clamp_page(page, page_size);
    Ok(repository::collections::list_paged(
        &state.db,
        page_size,
        (page - 1) * page_size,
        search.as_deref(),
    )
    .await?)
}

#[tauri::command]
pub async fn get_collection(state: State<'_, AppState>, cid: i64) -> CmdResult<Collection> {
    Ok(repository::collections::get(&state.db, cid).await?)
}

#[tauri::command]
pub async fn create_collection(
    state: State<'_, AppState>,
    payload: CreateCollection,
) -> CmdResult<i64> {
    // 新建合集属于作者本人，用「合集(自己)」权限
    let operator = require_feature(&state.db, &state.auth, FEATURE_COLLECTION_OWN).await?;
    Ok(repository::collections::create(&state.db, &payload, operator.id).await?)
}

#[tauri::command]
pub async fn update_collection(
    state: State<'_, AppState>,
    cid: i64,
    payload: CreateCollection,
) -> CmdResult<()> {
    let col = repository::collections::get(&state.db, cid).await?;
    let _user = require_collection_write(&state, col.user_id).await?;
    repository::collections::update(&state.db, cid, &payload).await?;
    Ok(())
}

#[tauri::command]
pub async fn delete_collection(state: State<'_, AppState>, cid: i64) -> CmdResult<()> {
    let col = repository::collections::get(&state.db, cid).await?;
    let _user = require_collection_write(&state, col.user_id).await?;
    repository::collections::soft_delete(&state.db, cid).await?;
    Ok(())
}

#[tauri::command]
pub async fn restore_collection(state: State<'_, AppState>, cid: i64) -> CmdResult<()> {
    let col = repository::collections::get(&state.db, cid).await?;
    let _user = require_collection_write(&state, col.user_id).await?;
    repository::collections::restore(&state.db, cid).await?;
    Ok(())
}

#[tauri::command]
pub async fn permanent_delete_collection(
    state: State<'_, AppState>,
    cid: i64,
) -> CmdResult<()> {
    let col = repository::collections::get(&state.db, cid).await?;
    let _user = require_collection_write(&state, col.user_id).await?;
    // 清理缩略图
    let storage = state.storage.get().await;
    if !col.thumbnail_path.is_empty() {
        let _ = storage.delete(&col.thumbnail_path).await;
    }
    repository::collections::permanent_delete(&state.db, cid).await?;
    Ok(())
}

#[tauri::command]
pub async fn sync_collection_resources(
    state: State<'_, AppState>,
    cid: i64,
    resource_ids: Vec<i64>,
) -> CmdResult<()> {
    let col = repository::collections::get(&state.db, cid).await?;
    let _user = require_collection_write(&state, col.user_id).await?;
    repository::collections::sync_resources(&state.db, cid, &resource_ids).await?;
    Ok(())
}

#[tauri::command]
pub async fn list_collection_resource_ids(
    state: State<'_, AppState>,
    cid: i64,
) -> CmdResult<Vec<i64>> {
    Ok(repository::collections::list_resource_ids(&state.db, cid).await?)
}
