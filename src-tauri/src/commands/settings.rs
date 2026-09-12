// 设置命令
use std::collections::HashMap;
use tauri::State;

use crate::auth::require_admin;
use crate::db::repository;
use crate::error::CmdResult;
use crate::state::AppState;

#[tauri::command]
pub async fn get_public_settings(state: State<'_, AppState>) -> CmdResult<HashMap<String, String>> {
    let all = repository::settings::get_all(&state.db).await?;
    let mut public = HashMap::new();
    if let Some(v) = all.get("site_name") {
        public.insert("site_name".into(), v.clone());
    }
    // 功能权限阈值对前端公开（仅用于 UI 显隐，真正校验在后端）
    for key in [
        "perm_article",
        "perm_collection",
        "perm_tag",
        "perm_trash",
        "perm_user",
    ] {
        public.insert(key.into(), all.get(key).cloned().unwrap_or_else(|| "5".into()));
    }
    Ok(public)
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> CmdResult<HashMap<String, String>> {
    let _user = require_admin(&state.auth).await?;
    Ok(repository::settings::get_all(&state.db).await?)
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    settings: HashMap<String, String>,
) -> CmdResult<()> {
    let _user = require_admin(&state.auth).await?;
    // 校验权限阈值
    for key in [
        "perm_article",
        "perm_collection",
        "perm_tag",
        "perm_trash",
        "perm_user",
    ] {
        if let Some(v) = settings.get(key) {
            let n: i64 = v.parse().map_err(|_| {
                crate::error::AppError::BadRequest("权限值必须为 0-5 的数字".into())
            })?;
            if !(0..=5).contains(&n) {
                return Err(crate::error::AppError::BadRequest("权限值必须为 0-5".into()));
            }
        }
    }
    repository::settings::set_many(&state.db, &settings).await?;
    // 如果存储相关设置变更，重置存储单例
    if settings.contains_key("local_upload_dir") || settings.contains_key("local_url_prefix") {
        let new_storage = crate::storage::get_storage(&state.db, &state.data_dir)
            .await
            .map_err(crate::error::AppError::Storage)?;
        state.reset_storage(new_storage).await;
    }
    Ok(())
}

/// 返回上传根目录的绝对路径（供前端用 convertFileSrc 将其拼接为 asset URL）
#[tauri::command]
pub async fn get_upload_base_dir(state: State<'_, AppState>) -> CmdResult<String> {
    Ok(state.upload_base_dir.to_string_lossy().to_string())
}
