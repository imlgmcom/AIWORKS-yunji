// 图片命令
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::auth::{require_feature, FEATURE_ARTICLE, FEATURE_COLLECTION};
use crate::db::repository;
use crate::error::CmdResult;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct UploadResult {
    pub id: i64,
    pub file_path: String,
    pub url: String,
    pub width: i64,
    pub height: i64,
}

// --- 资源图片 ---
#[tauri::command]
pub async fn list_resource_images(
    state: State<'_, AppState>,
    rid: i64,
) -> CmdResult<Vec<repository::images::Image>> {
    Ok(repository::images::list_for_resource(&state.db, rid).await?)
}

#[tauri::command]
pub async fn upload_resource_image(
    state: State<'_, AppState>,
    rid: i64,
    file_path: String,
) -> CmdResult<UploadResult> {
    let _user = require_feature(&state.db, &state.auth, FEATURE_ARTICLE).await?;
    let storage = state.storage.get().await;

    // 读取文件
    let data = std::fs::read(&file_path).map_err(|e| crate::error::AppError::Io(e))?;
    let bytes = bytes::Bytes::from(data);

    // 生成缩略图 + 获取尺寸
    let (width, height) = get_image_dimensions(&bytes);

    // 保存
    let filename = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("image.jpg");
    let saved_path = storage
        .save(bytes, filename, "gallery")
        .await
        .map_err(crate::error::AppError::Storage)?;
    let url = storage.get_url(&saved_path);
    let id = repository::images::add_to_resource(&state.db, rid, &saved_path, width, height).await?;

    Ok(UploadResult { id, file_path: saved_path, url, width, height })
}

#[tauri::command]
pub async fn delete_resource_image(
    state: State<'_, AppState>,
    img_id: i64,
) -> CmdResult<()> {
    let _user = require_feature(&state.db, &state.auth, FEATURE_ARTICLE).await?;
    let storage = state.storage.get().await;
    let path = repository::images::delete(&state.db, img_id).await?;
    let _ = storage.delete(&path).await;
    Ok(())
}

#[tauri::command]
pub async fn reorder_resource_images(
    state: State<'_, AppState>,
    rid: i64,
    image_ids: Vec<i64>,
) -> CmdResult<()> {
    let _user = require_feature(&state.db, &state.auth, FEATURE_ARTICLE).await?;
    repository::images::reorder_for_resource(&state.db, rid, &image_ids).await?;
    Ok(())
}

// --- 缩略图 ---
#[derive(Debug, Deserialize)]
pub struct SetThumbnailPayload {
    pub file_path: String,
}

#[tauri::command]
pub async fn set_resource_thumbnail(
    state: State<'_, AppState>,
    rid: i64,
    payload: SetThumbnailPayload,
) -> CmdResult<String> {
    let _user = require_feature(&state.db, &state.auth, FEATURE_ARTICLE).await?;
    let storage = state.storage.get().await;

    // 读取旧缩略图路径
    let old = repository::resources::get(&state.db, rid).await.ok();
    let old_path = old.as_ref().map(|r| r.thumbnail_path.clone()).unwrap_or_default();

    // 读取新文件
    let data = std::fs::read(&payload.file_path).map_err(crate::error::AppError::Io)?;
    let bytes = bytes::Bytes::from(data);
    let filename = std::path::Path::new(&payload.file_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("thumb.jpg");
    let saved = storage
        .save(bytes, filename, "thumbnails")
        .await
        .map_err(crate::error::AppError::Storage)?;

    // 更新数据库
    sqlx::query("UPDATE resources SET thumbnail_path=?, updated_at=datetime('now','localtime') WHERE id=?")
        .bind(&saved)
        .bind(rid)
        .execute(&state.db)
        .await
        .map_err(crate::error::AppError::Db)?;

    // 删除旧缩略图
    if !old_path.is_empty() {
        let _ = storage.delete(&old_path).await;
    }

    // 返回相对路径（与 gallery 上传一致），前端用 assetUrl 转 URL
    Ok(saved)
}

#[tauri::command]
pub async fn clear_resource_thumbnail(
    state: State<'_, AppState>,
    rid: i64,
) -> CmdResult<()> {
    let _user = require_feature(&state.db, &state.auth, FEATURE_ARTICLE).await?;
    let storage = state.storage.get().await;

    let r = repository::resources::get(&state.db, rid).await?;
    if !r.thumbnail_path.is_empty() {
        let _ = storage.delete(&r.thumbnail_path).await;
        sqlx::query("UPDATE resources SET thumbnail_path='', updated_at=datetime('now','localtime') WHERE id=?")
            .bind(rid)
            .execute(&state.db)
            .await
            .map_err(crate::error::AppError::Db)?;
    }
    Ok(())
}

// --- 合集图片 ---
#[tauri::command]
pub async fn list_collection_images(
    state: State<'_, AppState>,
    cid: i64,
) -> CmdResult<Vec<repository::images::Image>> {
    Ok(repository::images::list_for_collection(&state.db, cid).await?)
}

#[tauri::command]
pub async fn upload_collection_image(
    state: State<'_, AppState>,
    cid: i64,
    file_path: String,
) -> CmdResult<UploadResult> {
    let _user = require_feature(&state.db, &state.auth, FEATURE_COLLECTION).await?;
    let storage = state.storage.get().await;

    let data = std::fs::read(&file_path).map_err(crate::error::AppError::Io)?;
    let bytes = bytes::Bytes::from(data);
    let (width, height) = get_image_dimensions(&bytes);
    let filename = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("image.jpg");
    let saved = storage
        .save(bytes, filename, "gallery")
        .await
        .map_err(crate::error::AppError::Storage)?;
    let url = storage.get_url(&saved);
    let id = repository::images::add_to_collection(&state.db, cid, &saved, width, height).await?;

    Ok(UploadResult { id, file_path: saved, url, width, height })
}

#[tauri::command]
pub async fn delete_collection_image(
    state: State<'_, AppState>,
    img_id: i64,
) -> CmdResult<()> {
    let _user = require_feature(&state.db, &state.auth, FEATURE_COLLECTION).await?;
    let storage = state.storage.get().await;
    let path = repository::images::delete_collection_image(&state.db, img_id).await?;
    let _ = storage.delete(&path).await;
    Ok(())
}

#[tauri::command]
pub async fn reorder_collection_images(
    state: State<'_, AppState>,
    cid: i64,
    image_ids: Vec<i64>,
) -> CmdResult<()> {
    let _user = require_feature(&state.db, &state.auth, FEATURE_COLLECTION).await?;
    repository::images::reorder_for_collection(&state.db, cid, &image_ids).await?;
    Ok(())
}

// --- 工具函数 ---
fn get_image_dimensions(data: &[u8]) -> (i64, i64) {
    match image::load_from_memory(data) {
        Ok(img) => (img.width() as i64, img.height() as i64),
        Err(_) => (0, 0),
    }
}
