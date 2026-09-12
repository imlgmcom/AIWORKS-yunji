// 合集命令
use tauri::State;

use crate::auth::{require_feature, FEATURE_COLLECTION};
use crate::db::repository;
use crate::error::CmdResult;
use crate::state::AppState;

pub use repository::collections::{Collection, CreateCollection};

#[tauri::command]
pub async fn list_collections(state: State<'_, AppState>) -> CmdResult<Vec<Collection>> {
    Ok(repository::collections::list(&state.db).await?)
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
    let _user = require_feature(&state.db, &state.auth, FEATURE_COLLECTION).await?;
    Ok(repository::collections::create(&state.db, &payload).await?)
}

#[tauri::command]
pub async fn update_collection(
    state: State<'_, AppState>,
    cid: i64,
    payload: CreateCollection,
) -> CmdResult<()> {
    let _user = require_feature(&state.db, &state.auth, FEATURE_COLLECTION).await?;
    repository::collections::update(&state.db, cid, &payload).await?;
    Ok(())
}

#[tauri::command]
pub async fn delete_collection(state: State<'_, AppState>, cid: i64) -> CmdResult<()> {
    let _user = require_feature(&state.db, &state.auth, FEATURE_COLLECTION).await?;
    repository::collections::soft_delete(&state.db, cid).await?;
    Ok(())
}

#[tauri::command]
pub async fn restore_collection(state: State<'_, AppState>, cid: i64) -> CmdResult<()> {
    let _user = require_feature(&state.db, &state.auth, FEATURE_COLLECTION).await?;
    repository::collections::restore(&state.db, cid).await?;
    Ok(())
}

#[tauri::command]
pub async fn permanent_delete_collection(
    state: State<'_, AppState>,
    cid: i64,
) -> CmdResult<()> {
    let _user = require_feature(&state.db, &state.auth, FEATURE_COLLECTION).await?;
    // 清理缩略图
    let storage = state.storage.get().await;
    let col = repository::collections::get(&state.db, cid).await.ok();
    if let Some(c) = &col {
        if !c.thumbnail_path.is_empty() {
            let _ = storage.delete(&c.thumbnail_path).await;
        }
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
    let _user = require_feature(&state.db, &state.auth, FEATURE_COLLECTION).await?;
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
