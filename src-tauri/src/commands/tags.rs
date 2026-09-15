// 标签命令
use tauri::State;

use crate::auth::{require_feature, FEATURE_TAG};
use crate::db::repository;
use crate::error::CmdResult;
use crate::state::AppState;

pub use repository::tags::Tag;

#[tauri::command]
pub async fn list_tags(state: State<'_, AppState>) -> CmdResult<Vec<Tag>> {
    Ok(repository::tags::list(&state.db).await?)
}

#[tauri::command]
pub async fn list_tags_page(
    state: State<'_, AppState>,
    page: i64,
    page_size: i64,
    search: Option<String>,
) -> CmdResult<repository::Paged<Tag>> {
    let (page, page_size) = repository::clamp_page(page, page_size);
    Ok(repository::tags::list_paged(
        &state.db,
        page_size,
        (page - 1) * page_size,
        search.as_deref(),
    )
    .await?)
}

#[tauri::command]
pub async fn create_tag(state: State<'_, AppState>, name: String) -> CmdResult<i64> {
    let _user = require_feature(&state.db, &state.auth, FEATURE_TAG).await?;
    Ok(repository::tags::create(&state.db, &name).await?)
}

#[tauri::command]
pub async fn rename_tag(state: State<'_, AppState>, tag_id: i64, name: String) -> CmdResult<()> {
    let _user = require_feature(&state.db, &state.auth, FEATURE_TAG).await?;
    repository::tags::rename(&state.db, tag_id, &name).await?;
    Ok(())
}

#[tauri::command]
pub async fn delete_tag(state: State<'_, AppState>, tag_id: i64) -> CmdResult<()> {
    let _user = require_feature(&state.db, &state.auth, FEATURE_TAG).await?;
    repository::tags::delete(&state.db, tag_id).await?;
    Ok(())
}

#[tauri::command]
pub async fn merge_tags(
    state: State<'_, AppState>,
    source_ids: Vec<i64>,
    target_id: i64,
) -> CmdResult<()> {
    let _user = require_feature(&state.db, &state.auth, FEATURE_TAG).await?;
    repository::tags::merge(&state.db, &source_ids, target_id).await?;
    Ok(())
}
