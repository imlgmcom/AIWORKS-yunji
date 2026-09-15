// 设置命令
use std::collections::HashMap;
use std::path::Path;
use tauri::State;

use crate::auth::require_admin;
use crate::db::repository;
use crate::error::{AppError, CmdResult};
use crate::state::AppState;

/// 界面类设置（对所有用户公开，仅影响显示/窗口行为）
const PUBLIC_KEYS: &[&str] = &[
    "close_to_tray",
    "logo_mode",
    "logo_image_path",
    "logo_icon_path",
];

#[tauri::command]
pub async fn get_public_settings(state: State<'_, AppState>) -> CmdResult<HashMap<String, String>> {
    let all = repository::settings::get_all(&state.db).await?;
    let mut public = HashMap::new();
    if let Some(v) = all.get("site_name") {
        public.insert("site_name".into(), v.clone());
    }
    // 界面设置默认值
    let defaults = [
        ("close_to_tray", "0"),
        ("logo_mode", "text"),
        ("logo_image_path", ""),
        ("logo_icon_path", ""),
    ];
    for key in PUBLIC_KEYS {
        let default = defaults.iter().find(|(k, _)| k == key).map(|(_, d)| *d).unwrap_or("");
        public.insert((*key).into(), all.get(*key).cloned().unwrap_or_else(|| default.into()));
    }
    // 功能权限阈值对前端公开（仅用于 UI 显隐，真正校验在后端）
    for key in [
        "perm_article_own",
        "perm_article",
        "perm_collection_own",
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
    mut settings: HashMap<String, String>,
) -> CmdResult<()> {
    let _user = require_admin(&state.auth).await?;
    // 校验权限阈值
    for key in [
        "perm_article_own",
        "perm_article",
        "perm_collection_own",
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
    // LOGO 模式校验
    if let Some(mode) = settings.get("logo_mode") {
        if !["text", "icon_text", "image"].contains(&mode.as_str()) {
            return Err(AppError::BadRequest("LOGO 模式非法".into()));
        }
    }
    // 布尔开关归一化
    for key in ["close_to_tray"] {
        if let Some(v) = settings.get(key) {
            let on = matches!(v.to_lowercase().as_str(), "1" | "true" | "on" | "yes");
            settings.insert(key.into(), if on { "1".into() } else { "0".into() });
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

/// 上传品牌图片（仅管理员）。
/// kind = "image"：图片 LOGO（铺满左上角）；kind = "icon"：图标+文字模式的图标。
/// 固定保存到 logo 子目录，替换并删除旧图。
#[tauri::command]
pub async fn upload_logo(
    state: State<'_, AppState>,
    file_path: String,
    kind: String,
) -> CmdResult<String> {
    let _user = require_admin(&state.auth).await?;

    let (setting_key, file_stem) = match kind.as_str() {
        "image" => ("logo_image_path", "logo"),
        "icon" => ("logo_icon_path", "icon"),
        _ => return Err(AppError::BadRequest("kind 必须为 image 或 icon".into())),
    };

    let ext = Path::new(&file_path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    if !["jpg", "jpeg", "png", "webp", "gif", "svg", "ico"].contains(&ext.as_str()) {
        return Err(AppError::BadRequest("LOGO 仅支持 jpg/png/webp/gif/svg/ico".into()));
    }

    let data = tokio::fs::read(&file_path)
        .await
        .map_err(AppError::Io)?;
    let storage = state.storage.get().await;

    // 删除旧图
    if let Some(old) = repository::settings::get(&state.db, setting_key).await? {
        if !old.is_empty() {
            let _ = storage.delete(&old).await;
        }
    }

    let saved = storage
        .save_named(
            bytes::Bytes::from(data),
            &format!("{}.{}", file_stem, ext),
            "logo",
        )
        .await
        .map_err(AppError::Storage)?;
    repository::settings::set(&state.db, setting_key, &saved).await?;
    Ok(saved)
}

/// 清除品牌图片（仅管理员）。kind = "image" | "icon"
#[tauri::command]
pub async fn clear_logo(state: State<'_, AppState>, kind: String) -> CmdResult<()> {
    let _user = require_admin(&state.auth).await?;
    let setting_key = match kind.as_str() {
        "image" => "logo_image_path",
        "icon" => "logo_icon_path",
        _ => return Err(AppError::BadRequest("kind 必须为 image 或 icon".into())),
    };
    let storage = state.storage.get().await;
    if let Some(old) = repository::settings::get(&state.db, setting_key).await? {
        if !old.is_empty() {
            let _ = storage.delete(&old).await;
        }
    }
    repository::settings::set(&state.db, setting_key, "").await?;
    Ok(())
}
