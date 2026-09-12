// 图片 Repository（资源图集 + 合集图集）
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: i64,
    pub file_path: String,
    pub sort_order: i64,
    pub width: i64,
    pub height: i64,
    pub created_at: String,
}

// --- 资源图片 ---
pub async fn list_for_resource(pool: &SqlitePool, rid: i64) -> Result<Vec<Image>, AppError> {
    let rows = sqlx::query_as::<_, (i64, String, i64, i64, i64, String)>(
        "SELECT id, file_path, sort_order, width, height, created_at \
         FROM images WHERE resource_id=? ORDER BY sort_order, id",
    )
    .bind(rid)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, file_path, sort_order, width, height, created_at)| Image {
            id, file_path, sort_order, width, height, created_at,
        })
        .collect())
}

pub async fn add_to_resource(pool: &SqlitePool, rid: i64, file_path: &str, width: i64, height: i64) -> Result<i64, AppError> {
    let max_order: Option<i64> = sqlx::query_scalar("SELECT MAX(sort_order) FROM images WHERE resource_id=?")
        .bind(rid)
        .fetch_optional(pool)
        .await?;
    let sort_order = max_order.unwrap_or(-1) + 1;
    let result = sqlx::query(
        "INSERT INTO images (resource_id, file_path, sort_order, width, height) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(rid)
    .bind(file_path)
    .bind(sort_order)
    .bind(width)
    .bind(height)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn delete(pool: &SqlitePool, img_id: i64) -> Result<String, AppError> {
    let row = sqlx::query_as::<_, (String,)>("SELECT file_path FROM images WHERE id=?")
        .bind(img_id)
        .fetch_optional(pool)
        .await?;
    if let Some((path,)) = row {
        sqlx::query("DELETE FROM images WHERE id=?")
            .bind(img_id)
            .execute(pool)
            .await?;
        Ok(path)
    } else {
        Err(AppError::NotFound)
    }
}

pub async fn reorder_for_resource(pool: &SqlitePool, rid: i64, image_ids: &[i64]) -> Result<(), AppError> {
    for (i, &img_id) in image_ids.iter().enumerate() {
        sqlx::query("UPDATE images SET sort_order=? WHERE id=? AND resource_id=?")
            .bind(i as i64)
            .bind(img_id)
            .bind(rid)
            .execute(pool)
            .await?;
    }
    Ok(())
}

// --- 合集图片 ---
pub async fn list_for_collection(pool: &SqlitePool, cid: i64) -> Result<Vec<Image>, AppError> {
    let rows = sqlx::query_as::<_, (i64, String, i64, i64, i64, String)>(
        "SELECT id, file_path, sort_order, width, height, created_at \
         FROM collection_images WHERE collection_id=? ORDER BY sort_order, id",
    )
    .bind(cid)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, file_path, sort_order, width, height, created_at)| Image {
            id, file_path, sort_order, width, height, created_at,
        })
        .collect())
}

pub async fn add_to_collection(pool: &SqlitePool, cid: i64, file_path: &str, width: i64, height: i64) -> Result<i64, AppError> {
    let max_order: Option<i64> = sqlx::query_scalar("SELECT MAX(sort_order) FROM collection_images WHERE collection_id=?")
        .bind(cid)
        .fetch_optional(pool)
        .await?;
    let sort_order = max_order.unwrap_or(-1) + 1;
    let result = sqlx::query(
        "INSERT INTO collection_images (collection_id, file_path, sort_order, width, height) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(cid)
    .bind(file_path)
    .bind(sort_order)
    .bind(width)
    .bind(height)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn delete_collection_image(pool: &SqlitePool, img_id: i64) -> Result<String, AppError> {
    let row = sqlx::query_as::<_, (String,)>("SELECT file_path FROM collection_images WHERE id=?")
        .bind(img_id)
        .fetch_optional(pool)
        .await?;
    if let Some((path,)) = row {
        sqlx::query("DELETE FROM collection_images WHERE id=?")
            .bind(img_id)
            .execute(pool)
            .await?;
        Ok(path)
    } else {
        Err(AppError::NotFound)
    }
}

pub async fn reorder_for_collection(pool: &SqlitePool, cid: i64, image_ids: &[i64]) -> Result<(), AppError> {
    for (i, &img_id) in image_ids.iter().enumerate() {
        sqlx::query("UPDATE collection_images SET sort_order=? WHERE id=? AND collection_id=?")
            .bind(i as i64)
            .bind(img_id)
            .bind(cid)
            .execute(pool)
            .await?;
    }
    Ok(())
}
